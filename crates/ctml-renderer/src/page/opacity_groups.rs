use super::PageRasterizer;
use crate::style::{BackgroundBlendMode, WritingMode};
use std::collections::HashSet;

#[derive(Clone, Copy)]
enum PaintEvent {
    Run(usize),
    Element(usize),
}

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
            false,
            None,
            true,
        );
        if self.root_opacity == 0.0 {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[..3].copy_from_slice(&[255, 255, 255]);
            }
            return pixels;
        }
        if self.root_style.mix_blend_mode != BackgroundBlendMode::Normal {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[..3].copy_from_slice(&[255, 255, 255]);
                pixel[3] = 255;
            }
            let mut source = self.render_scoped_raw(
                None,
                [0; 3],
                width,
                height,
                writing_mode,
                true,
                false,
                true,
                None,
                true,
            );
            for event in self.paint_events(None) {
                match event {
                    PaintEvent::Run(index) => {
                        self.composite_run(index, None, &mut source, width, height, writing_mode);
                    }
                    PaintEvent::Element(node_id) => {
                        self.composite_child(node_id, &mut source, width, height, writing_mode);
                    }
                }
            }
            composite_source_layer(
                &mut pixels,
                &source,
                self.root_opacity,
                self.root_style.mix_blend_mode,
                width,
                height,
            );
            return pixels;
        }
        for event in self.paint_events(None) {
            match event {
                PaintEvent::Run(index) => {
                    self.composite_run(index, None, &mut pixels, width, height, writing_mode);
                }
                PaintEvent::Element(node_id) => {
                    self.composite_child(node_id, &mut pixels, width, height, writing_mode);
                }
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
    ) -> Vec<u8> {
        self.render_group_content(node_id, width, height, writing_mode)
    }

    fn render_group_content(
        &mut self,
        node_id: usize,
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) -> Vec<u8> {
        let mut pixels = self.render_scoped_raw(
            Some(node_id),
            [0; 3],
            width,
            height,
            writing_mode,
            false,
            true,
            true,
            None,
            true,
        );
        for event in self.paint_events(Some(node_id)) {
            match event {
                PaintEvent::Run(index) => self.composite_run(
                    index,
                    Some(node_id),
                    &mut pixels,
                    width,
                    height,
                    writing_mode,
                ),
                PaintEvent::Element(child) => {
                    self.composite_child(child, &mut pixels, width, height, writing_mode);
                }
            }
        }
        pixels
    }

    fn composite_child(
        &mut self,
        node_id: usize,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) {
        if let Some(opacity) = self.opacity_groups.get(&node_id).copied() {
            if opacity == 0.0 {
                return;
            }
            if opacity < 1.0 {
                let source = self.render_group_layer(node_id, width, height, writing_mode);
                composite_source_layer(
                    pixels,
                    &source,
                    opacity,
                    self.mix_blend_modes[&node_id],
                    width,
                    height,
                );
                return;
            }
        }
        self.composite_node_layer(node_id, pixels, width, height, writing_mode);
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
        transparent: bool,
        selected_run: Option<usize>,
        boxes_only: bool,
    ) -> Vec<u8> {
        let original_runs = std::mem::take(&mut self.runs);
        let original_cursor_spans = std::mem::take(&mut self.cursor_spans);
        let original_decoration_opacity_scales =
            std::mem::take(&mut self.decoration_opacity_scales);
        self.runs = original_runs.clone();
        self.decoration_opacity_scales = Vec::with_capacity(self.runs.len());
        for (run_index, run) in self.runs.iter_mut().enumerate() {
            let original_opacity = run.style.opacity;
            let scope_index = scope.and_then(|node_id| {
                run.element_path
                    .iter()
                    .position(|path_node| *path_node == node_id)
            });
            run.style.opacity = if let Some(selected_run) = selected_run {
                if selected_run == run_index { 1.0 } else { 0.0 }
            } else if boxes_only {
                if scope_index.is_some() || (scope.is_none() && run.element_path.is_empty()) {
                    1.0
                } else {
                    0.0
                }
            } else {
                match scope_index {
                    Some(scope_index) if !own_only || scope_index + 1 == run.element_path.len() => {
                        1.0
                    }
                    Some(_) => 0.0,
                    None if scope.is_some() || (root_only && !run.element_path.is_empty()) => 0.0,
                    None if has_opacity_group(&run.element_path, &self.opacity_groups) => 0.0,
                    None => 1.0,
                }
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
                let visible_in_scope = if selected_run.is_some() {
                    false
                } else {
                    match scope_index {
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
                        None => !has_opacity_group(
                            &run.element_path[..=size_index],
                            &self.opacity_groups,
                        ),
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
            transparent,
            boxes_only,
        );
        self.runs = original_runs;
        self.cursor_spans = original_cursor_spans;
        self.decoration_opacity_scales = original_decoration_opacity_scales;
        pixels
    }

    fn composite_run(
        &mut self,
        run_index: usize,
        scope: Option<usize>,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) {
        let source = self.render_scoped_raw(
            scope,
            [0; 3],
            width,
            height,
            writing_mode,
            false,
            false,
            true,
            Some(run_index),
            false,
        );
        composite_source_layer(
            pixels,
            &source,
            1.0,
            BackgroundBlendMode::Normal,
            width,
            height,
        );
    }

    fn composite_node_layer(
        &mut self,
        node_id: usize,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) {
        let isolated = self.isolation_groups.contains(&node_id)
            || self.mix_blend_modes[&node_id] != BackgroundBlendMode::Normal;
        if isolated {
            let source = self.render_group_layer(node_id, width, height, writing_mode);
            composite_source_layer(
                pixels,
                &source,
                1.0,
                self.mix_blend_modes[&node_id],
                width,
                height,
            );
            return;
        }

        let source = self.render_scoped_raw(
            Some(node_id),
            [0; 3],
            width,
            height,
            writing_mode,
            false,
            true,
            true,
            None,
            true,
        );
        composite_source_layer(
            pixels,
            &source,
            1.0,
            BackgroundBlendMode::Normal,
            width,
            height,
        );
        for event in self.paint_events(Some(node_id)) {
            match event {
                PaintEvent::Run(index) => {
                    self.composite_run(index, Some(node_id), pixels, width, height, writing_mode)
                }
                PaintEvent::Element(child) => {
                    self.composite_child(child, pixels, width, height, writing_mode);
                }
            }
        }
    }

    fn paint_events(&self, scope: Option<usize>) -> Vec<PaintEvent> {
        let mut seen_elements = HashSet::new();
        let mut events = Vec::new();
        for (run_index, run) in self.runs.iter().enumerate() {
            let start = if let Some(scope) = scope {
                let Some(index) = run
                    .element_path
                    .iter()
                    .position(|node_id| *node_id == scope)
                else {
                    continue;
                };
                index + 1
            } else {
                0
            };
            if let Some(element) = run.element_path.get(start).copied() {
                if seen_elements.insert(element) {
                    events.push(PaintEvent::Element(element));
                }
            } else if scope.is_none_or(|scope| run.element_path.last() == Some(&scope)) {
                events.push(PaintEvent::Run(run_index));
            }
        }
        events
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

fn composite_source_layer(
    destination: &mut [u8],
    source: &[u8],
    opacity: f32,
    blend_mode: BackgroundBlendMode,
    width: u32,
    height: u32,
) {
    super::background_blend::composite_source_layer(
        destination,
        source,
        opacity,
        blend_mode,
        width,
        height,
    );
}
