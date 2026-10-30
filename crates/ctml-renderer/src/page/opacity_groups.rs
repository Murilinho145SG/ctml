use super::PageRasterizer;
use crate::style::WritingMode;
use std::collections::HashSet;

impl PageRasterizer {
    pub(super) fn render_layout_pixels(
        &mut self,
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> Vec<u8> {
        let raw_output = self.render_layout_pixels_raw(width, height, writing_mode);
        let mut output = raw_output.clone();
        let nested_groups = self.nested_opacity_groups();
        let mut groups = self
            .opacity_groups
            .iter()
            .filter(|(node_id, opacity)| {
                **opacity > 0.0 && **opacity < 1.0 && !nested_groups.contains(node_id)
            })
            .map(|(node_id, opacity)| (*node_id, *opacity))
            .collect::<Vec<_>>();
        groups.sort_by_key(|(node_id, _)| *node_id);

        let mut painted_group_mask = vec![false; raw_output.len() / 4];
        for (node_id, opacity) in groups {
            let without_group =
                self.render_with_opacity_override(node_id, 0.0, width, height, writing_mode);
            let full_group =
                self.render_with_opacity_override(node_id, 1.0, width, height, writing_mode);
            let mut group_mask = vec![false; painted_group_mask.len()];
            let mut overlaps_another_group = false;
            for ((without_group, full_group), (index, seen)) in without_group
                .chunks_exact(4)
                .zip(full_group.chunks_exact(4))
                .zip(painted_group_mask.iter().enumerate())
            {
                let affects_pixel = without_group[..3] != full_group[..3];
                group_mask[index] = affects_pixel;
                overlaps_another_group |= affects_pixel && *seen;
            }
            if overlaps_another_group {
                continue;
            }
            for (((output, raw), without_group), full_group) in output
                .chunks_exact_mut(4)
                .zip(raw_output.chunks_exact(4))
                .zip(without_group.chunks_exact(4))
                .zip(full_group.chunks_exact(4))
            {
                for channel in 0..3 {
                    let without = f32::from(without_group[channel]);
                    let full = f32::from(full_group[channel]);
                    let correctly_composited = without + opacity * (full - without);
                    output[channel] = (f32::from(output[channel]) + correctly_composited
                        - f32::from(raw[channel]))
                    .round()
                    .clamp(0.0, 255.0) as u8;
                }
            }
            for (seen, affects_pixel) in painted_group_mask.iter_mut().zip(group_mask) {
                *seen |= affects_pixel;
            }
        }
        output
    }

    fn nested_opacity_groups(&self) -> HashSet<usize> {
        let mut nested_groups = HashSet::new();
        for run in &self.runs {
            let groups = run
                .element_path
                .iter()
                .filter(|node_id| {
                    self.opacity_groups
                        .get(node_id)
                        .is_some_and(|opacity| *opacity > 0.0 && *opacity < 1.0)
                })
                .copied()
                .collect::<Vec<_>>();
            if groups.len() > 1 {
                nested_groups.extend(groups);
            }
        }
        nested_groups
    }

    fn render_with_opacity_override(
        &mut self,
        node_id: usize,
        opacity: f32,
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> Vec<u8> {
        let original_runs = std::mem::take(&mut self.runs);
        let original_cursor_spans = std::mem::take(&mut self.cursor_spans);
        self.runs = original_runs.clone();
        for run in &mut self.runs {
            let Some(group_index) = run.element_path.iter().position(|id| *id == node_id) else {
                continue;
            };
            run.style.opacity =
                opacity_for_path(&run.element_path, &self.opacity_groups, node_id, opacity);
            for size in &mut run.style.layout_sizes {
                let Some(size_index) = run.element_path.iter().position(|id| *id == size.node_id)
                else {
                    continue;
                };
                if size_index >= group_index {
                    size.opacity = opacity_for_path(
                        &run.element_path[..=size_index],
                        &self.opacity_groups,
                        node_id,
                        opacity,
                    );
                }
            }
            if let Some(decoration) = &mut run.first_line_decoration {
                adjust_decoration_opacity(decoration, self.opacity_groups[&node_id], opacity);
            }
            if let Some(decoration) = &mut run.first_letter_decoration {
                adjust_decoration_opacity(decoration, self.opacity_groups[&node_id], opacity);
            }
        }
        self.cursor_spans = super::cursor_spans::cursor_spans(&self.runs);
        let pixels = self.render_layout_pixels_raw(width, height, writing_mode);
        self.runs = original_runs;
        self.cursor_spans = original_cursor_spans;
        pixels
    }
}

fn opacity_for_path(
    path: &[usize],
    opacity_groups: &std::collections::HashMap<usize, f32>,
    override_node: usize,
    override_opacity: f32,
) -> f32 {
    path.iter().fold(1.0, |opacity, node_id| {
        opacity
            * if *node_id == override_node {
                override_opacity
            } else {
                opacity_groups.get(node_id).copied().unwrap_or(1.0)
            }
    })
}

fn adjust_decoration_opacity(
    decoration: &mut crate::css_style::PseudoDecoration,
    original_opacity: f32,
    new_opacity: f32,
) {
    if original_opacity == 0.0 {
        decoration.color[3] = 0;
    } else {
        decoration.color[3] = (f32::from(decoration.color[3]) * new_opacity / original_opacity)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
}
