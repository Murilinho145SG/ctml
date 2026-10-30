use super::PageRasterizer;
use crate::style::WritingMode;
use std::collections::{HashMap, HashSet};

impl PageRasterizer {
    pub(super) fn render_layout_pixels(
        &mut self,
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> Vec<u8> {
        self.render_composited_scope(&HashMap::new(), None, width, height, writing_mode)
    }

    fn render_composited_scope(
        &mut self,
        overrides: &HashMap<usize, f32>,
        scope: Option<usize>,
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> Vec<u8> {
        let raw_output = self.render_with_opacity_overrides(overrides, width, height, writing_mode);
        let mut output = raw_output.clone();
        if scope.is_some_and(|node_id| overrides.get(&node_id) == Some(&0.0)) {
            return output;
        }

        let mut children = self
            .opacity_group_children(scope)
            .into_iter()
            .collect::<Vec<_>>();
        children.sort_unstable_by_key(|node_id| *node_id);
        let mut painted_group_mask = vec![false; raw_output.len() / 4];
        for node_id in children {
            let opacity = self.opacity_groups[&node_id];
            let without_overrides = with_opacity_override(overrides, node_id, 0.0);
            let full_overrides = with_opacity_override(overrides, node_id, 1.0);
            let without_group = self.render_composited_scope(
                &without_overrides,
                Some(node_id),
                width,
                height,
                writing_mode,
            );
            let full_group = self.render_composited_scope(
                &full_overrides,
                Some(node_id),
                width,
                height,
                writing_mode,
            );
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

    fn opacity_group_children(&self, scope: Option<usize>) -> HashSet<usize> {
        let mut children = HashSet::new();
        for run in &self.runs {
            let start = scope.map_or(0, |scope| {
                run.element_path
                    .iter()
                    .position(|node_id| *node_id == scope)
                    .map_or(run.element_path.len(), |index| index + 1)
            });
            if start == run.element_path.len() {
                continue;
            }
            if let Some(node_id) = run.element_path[start..].iter().find(|node_id| {
                self.opacity_groups
                    .get(node_id)
                    .is_some_and(|opacity| *opacity > 0.0 && *opacity < 1.0)
            }) {
                children.insert(*node_id);
            }
        }
        children
    }

    fn render_with_opacity_overrides(
        &mut self,
        overrides: &HashMap<usize, f32>,
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> Vec<u8> {
        let original_runs = std::mem::take(&mut self.runs);
        let original_cursor_spans = std::mem::take(&mut self.cursor_spans);
        self.runs = original_runs.clone();
        for run in &mut self.runs {
            let original_opacity =
                opacity_for_path(&run.element_path, &self.opacity_groups, &HashMap::new());
            let new_opacity = opacity_for_path(&run.element_path, &self.opacity_groups, overrides);
            let ratio = if original_opacity == 0.0 {
                0.0
            } else {
                new_opacity / original_opacity
            };
            run.style.opacity = new_opacity;
            for size in &mut run.style.layout_sizes {
                let Some(size_index) = run.element_path.iter().position(|id| *id == size.node_id)
                else {
                    continue;
                };
                size.opacity = opacity_for_path(
                    &run.element_path[..=size_index],
                    &self.opacity_groups,
                    overrides,
                );
            }
            if let Some(decoration) = &mut run.first_line_decoration {
                adjust_decoration_opacity(decoration, ratio);
            }
            if let Some(decoration) = &mut run.first_letter_decoration {
                adjust_decoration_opacity(decoration, ratio);
            }
        }
        self.cursor_spans = super::cursor_spans::cursor_spans(&self.runs);
        let pixels = self.render_layout_pixels_raw(width, height, writing_mode);
        self.runs = original_runs;
        self.cursor_spans = original_cursor_spans;
        pixels
    }
}

fn with_opacity_override(
    current: &HashMap<usize, f32>,
    node_id: usize,
    opacity: f32,
) -> HashMap<usize, f32> {
    let mut overrides = current.clone();
    overrides.insert(node_id, opacity);
    overrides
}

fn opacity_for_path(
    path: &[usize],
    opacity_groups: &HashMap<usize, f32>,
    overrides: &HashMap<usize, f32>,
) -> f32 {
    path.iter().fold(1.0, |opacity, node_id| {
        opacity
            * overrides
                .get(node_id)
                .or_else(|| opacity_groups.get(node_id))
                .copied()
                .unwrap_or(1.0)
    })
}

fn adjust_decoration_opacity(decoration: &mut crate::css_style::PseudoDecoration, ratio: f32) {
    decoration.color[3] = (f32::from(decoration.color[3]) * ratio)
        .round()
        .clamp(0.0, 255.0) as u8;
}
