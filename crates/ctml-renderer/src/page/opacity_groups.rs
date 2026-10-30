use super::PageRasterizer;
use crate::style::WritingMode;
use std::collections::HashSet;

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

impl PageRasterizer {
    pub(super) fn render_layout_pixels(
        &mut self,
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> Vec<u8> {
        let mut pixels = self.render_scoped_raw(
            None,
            self.root_style.background,
            width,
            height,
            writing_mode,
            true,
            false,
        );
        if self.root_opacity == 0.0 {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[..3].copy_from_slice(&WHITE);
            }
            return pixels;
        }
        for node_id in self.root_child_ids() {
            if let Some(opacity) = self.opacity_groups.get(&node_id).copied() {
                if opacity > 0.0 && opacity < 1.0 {
                    let (black, white) =
                        self.render_group_layer(node_id, width, height, writing_mode);
                    composite_group_layer(&mut pixels, &black, &white, opacity);
                } else if opacity == 0.0 {
                    continue;
                } else {
                    self.composite_node_layer(node_id, &mut pixels, width, height, writing_mode);
                }
            } else {
                self.composite_node_layer(node_id, &mut pixels, width, height, writing_mode);
            }
        }
        if self.root_opacity < 1.0 {
            for pixel in pixels.chunks_exact_mut(4) {
                for channel in &mut pixel[..3] {
                    *channel = (255.0 + (f32::from(*channel) - 255.0) * self.root_opacity)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        }
        pixels
    }

    fn render_group_layer(
        &mut self,
        node_id: usize,
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> (Vec<u8>, Vec<u8>) {
        let black = self.render_group_content(node_id, BLACK, width, height, writing_mode);
        let white = self.render_group_content(node_id, WHITE, width, height, writing_mode);
        (black, white)
    }

    fn render_group_content(
        &mut self,
        node_id: usize,
        background: [u8; 3],
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> Vec<u8> {
        let mut pixels = self.render_scoped_raw(
            Some(node_id),
            background,
            width,
            height,
            writing_mode,
            false,
            true,
        );
        for child in self.child_ids(node_id) {
            if let Some(opacity) = self.opacity_groups.get(&child).copied() {
                if opacity > 0.0 && opacity < 1.0 {
                    let (black, white) =
                        self.render_group_layer(child, width, height, writing_mode);
                    composite_group_layer(&mut pixels, &black, &white, opacity);
                } else if opacity == 0.0 {
                    continue;
                } else {
                    self.composite_node_layer(child, &mut pixels, width, height, writing_mode);
                }
            } else {
                self.composite_node_layer(child, &mut pixels, width, height, writing_mode);
            }
        }
        pixels
    }

    fn render_scoped_raw(
        &mut self,
        scope: Option<usize>,
        background: [u8; 3],
        width: u32,
        height: u32,
        writing_mode: WritingMode,
        root_only: bool,
        own_only: bool,
    ) -> Vec<u8> {
        let original_runs = std::mem::take(&mut self.runs);
        let original_cursor_spans = std::mem::take(&mut self.cursor_spans);
        let original_decoration_opacity_scales =
            std::mem::take(&mut self.decoration_opacity_scales);
        self.runs = original_runs.clone();
        self.decoration_opacity_scales = Vec::with_capacity(self.runs.len());
        for run in &mut self.runs {
            let original_opacity = run.style.opacity;
            let scope_index = scope.and_then(|node_id| {
                run.element_path
                    .iter()
                    .position(|path_node| *path_node == node_id)
            });
            run.style.opacity = match scope_index {
                Some(scope_index) if !own_only || scope_index + 1 == run.element_path.len() => 1.0,
                Some(_) => 0.0,
                None if scope.is_some() || (root_only && !run.element_path.is_empty()) => 0.0,
                None if has_opacity_group(&run.element_path, &self.opacity_groups) => 0.0,
                None => 1.0,
            };
            let opacity_scale = if original_opacity == 0.0 {
                0.0
            } else {
                run.style.opacity / original_opacity
            };
            self.decoration_opacity_scales.push(opacity_scale);
            for size in &mut run.style.layout_sizes {
                let Some(size_index) = run.element_path.iter().position(|id| *id == size.node_id)
                else {
                    size.opacity = 0.0;
                    continue;
                };
                let visible_in_scope = match scope_index {
                    Some(scope_index) if own_only => size_index == scope_index,
                    Some(scope_index) => {
                        size_index >= scope_index
                            && (size_index == scope_index
                                || !has_opacity_group(
                                    &run.element_path[scope_index + 1..=size_index],
                                    &self.opacity_groups,
                                ))
                    }
                    None if scope.is_some() || root_only => false,
                    None => {
                        !has_opacity_group(&run.element_path[..=size_index], &self.opacity_groups)
                    }
                };
                size.opacity = if visible_in_scope { 1.0 } else { 0.0 };
            }
            adjust_decoration_opacity(&mut run.first_line_decoration, opacity_scale);
            adjust_decoration_opacity(&mut run.first_letter_decoration, opacity_scale);
        }
        self.cursor_spans = super::cursor_spans::cursor_spans(&self.runs);
        let pixels = self.render_layout_pixels_raw(
            width,
            height,
            writing_mode,
            background,
            scope.is_some(),
            false,
        );
        self.runs = original_runs;
        self.cursor_spans = original_cursor_spans;
        self.decoration_opacity_scales = original_decoration_opacity_scales;
        pixels
    }

    fn composite_node_layer(
        &mut self,
        node_id: usize,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) {
        let black = self.render_scoped_raw(
            Some(node_id),
            BLACK,
            width,
            height,
            writing_mode,
            false,
            true,
        );
        let white = self.render_scoped_raw(
            Some(node_id),
            WHITE,
            width,
            height,
            writing_mode,
            false,
            true,
        );
        composite_group_layer(pixels, &black, &white, 1.0);
        for child in self.child_ids(node_id) {
            if let Some(opacity) = self.opacity_groups.get(&child).copied() {
                if opacity > 0.0 && opacity < 1.0 {
                    let (black, white) =
                        self.render_group_layer(child, width, height, writing_mode);
                    composite_group_layer(pixels, &black, &white, opacity);
                } else if opacity == 0.0 {
                    continue;
                } else {
                    self.composite_node_layer(child, pixels, width, height, writing_mode);
                }
            } else {
                self.composite_node_layer(child, pixels, width, height, writing_mode);
            }
        }
    }

    fn child_ids(&self, scope: usize) -> Vec<usize> {
        let mut children = HashSet::new();
        for run in &self.runs {
            if let Some(index) = run
                .element_path
                .iter()
                .position(|node_id| *node_id == scope)
                && let Some(child) = run.element_path.get(index + 1)
            {
                children.insert(*child);
            }
        }
        let mut children = children.into_iter().collect::<Vec<_>>();
        children.sort_unstable();
        children
    }

    fn root_child_ids(&self) -> Vec<usize> {
        let mut children = HashSet::new();
        for run in &self.runs {
            if let Some(node_id) = run.element_path.first() {
                children.insert(*node_id);
            }
        }
        let mut children = children.into_iter().collect::<Vec<_>>();
        children.sort_unstable();
        children
    }
}

fn has_opacity_group(
    path: &[usize],
    opacity_groups: &std::collections::HashMap<usize, f32>,
) -> bool {
    path.iter().any(|node_id| {
        opacity_groups
            .get(node_id)
            .is_some_and(|opacity| *opacity < 1.0)
    })
}

fn adjust_decoration_opacity(
    decoration: &mut Option<crate::css_style::PseudoDecoration>,
    opacity_scale: f32,
) {
    if let Some(decoration) = decoration {
        decoration.color[3] = (f32::from(decoration.color[3]) * opacity_scale)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
}

fn composite_group_layer(destination: &mut [u8], black: &[u8], white: &[u8], opacity: f32) {
    for ((destination, black), white) in destination
        .chunks_exact_mut(4)
        .zip(black.chunks_exact(4))
        .zip(white.chunks_exact(4))
    {
        let transmittance = (0..3)
            .map(|channel| (i16::from(white[channel]) - i16::from(black[channel])) as f32 / 255.0)
            .sum::<f32>()
            / 3.0;
        let group_alpha = (1.0 - transmittance).clamp(0.0, 1.0) * opacity;
        for channel in 0..3 {
            destination[channel] = (f32::from(black[channel]) * opacity
                + f32::from(destination[channel]) * (1.0 - group_alpha))
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
}
