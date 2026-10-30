use cosmic_text::{Buffer, FontSystem, Scroll};

use crate::style::WritingMode;

use super::{PageRasterizer, scrolling::scrollbar_axis};

const BAR_THICKNESS: f32 = 8.0;
const MIN_THUMB_LENGTH: f32 = 18.0;
const TRACK_COLOR: [u8; 4] = [232, 232, 232, 255];
const THUMB_COLOR: [u8; 4] = [142, 142, 142, 255];

#[derive(Clone, Copy)]
struct Scrollbar {
    horizontal: bool,
    content_extent: f32,
    viewport_extent: f32,
    offset: f32,
    always_visible: bool,
}

impl PageRasterizer {
    pub(super) fn draw_root_scrollbars(
        &mut self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) {
        let (content_block, content_inline) =
            content_extents(&mut self.buffer, &mut self.font_system);
        let scroll = self.buffer.scroll();
        let (block_offset, inline_offset) = scroll_offsets(&self.buffer, scroll);
        let (content_x, offset_x, content_y, offset_y) = match writing_mode {
            WritingMode::HorizontalTb => {
                (content_inline, inline_offset, content_block, block_offset)
            }
            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                (content_block, block_offset, content_inline, inline_offset)
            }
        };
        let viewport_width = (width as f32 - super::PAGE_PADDING * 2.0).max(0.0);
        let viewport_height = (height as f32 - super::PAGE_PADDING * 2.0).max(0.0);
        let scrollbars = [
            Scrollbar {
                horizontal: true,
                content_extent: content_x,
                viewport_extent: viewport_width,
                offset: offset_x,
                always_visible: false,
            },
            Scrollbar {
                horizontal: false,
                content_extent: content_y,
                viewport_extent: viewport_height,
                offset: offset_y,
                always_visible: false,
            },
        ];

        for (mut scrollbar, mode) in scrollbars.into_iter().zip([
            scrollbar_axis(self.root_style.overflow_x, self.root_style.overflow_y),
            scrollbar_axis(self.root_style.overflow_y, self.root_style.overflow_x),
        ]) {
            let Some(always_visible) = mode else {
                continue;
            };
            scrollbar.always_visible = always_visible;
            if !scrollbar.always_visible && scrollbar.content_extent <= scrollbar.viewport_extent {
                continue;
            }
            draw_scrollbar(pixels, width, height, scrollbar);
        }
        self.draw_nested_scrollbars(pixels, width, height, writing_mode);
    }

    fn draw_nested_scrollbars(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        writing_mode: WritingMode,
    ) {
        if writing_mode != WritingMode::HorizontalTb {
            return;
        }
        let mut seen = std::collections::HashSet::new();
        for run in &self.runs {
            for size in &run.style.layout_sizes {
                if !seen.insert(size.node_id) || size.writing_mode != WritingMode::HorizontalTb {
                    continue;
                }
                let Some(geometry) = self.overflow_geometry.get(&size.node_id) else {
                    continue;
                };
                let clip = geometry.clip;
                let (offset_x, offset_y) = self
                    .nested_scroll
                    .get(&size.node_id)
                    .copied()
                    .unwrap_or_default();
                if let Some(always_visible) = scrollbar_axis(size.overflow_x, size.overflow_y) {
                    let viewport = clip.right - clip.left;
                    if always_visible || geometry.content_inline_extent > viewport {
                        draw_horizontal_scrollbar(
                            pixels,
                            width,
                            height,
                            clip.left,
                            clip.bottom - BAR_THICKNESS,
                            viewport,
                            geometry.content_inline_extent,
                            offset_x,
                        );
                    }
                }
                if let Some(always_visible) = scrollbar_axis(size.overflow_y, size.overflow_x) {
                    let viewport = clip.bottom - clip.top;
                    if always_visible || geometry.content_block_extent > viewport {
                        draw_vertical_scrollbar(
                            pixels,
                            width,
                            height,
                            clip.right - BAR_THICKNESS,
                            clip.top,
                            viewport,
                            geometry.content_block_extent,
                            offset_y,
                        );
                    }
                }
            }
        }
    }
}

fn content_extents(buffer: &mut Buffer, font_system: &mut FontSystem) -> (f32, f32) {
    let metrics = buffer.metrics();
    let mut block_extent = 0.0;
    let mut inline_extent: f32 = 0.0;
    for line_index in 0..buffer.lines.len() {
        if let Some(layout) = buffer.line_layout(font_system, line_index) {
            for layout_line in layout {
                block_extent += layout_line.line_height_opt.unwrap_or(metrics.line_height);
                inline_extent = inline_extent.max(layout_line.w);
            }
        }
    }
    (block_extent, inline_extent)
}

fn scroll_offsets(buffer: &Buffer, scroll: Scroll) -> (f32, f32) {
    let block_offset = buffer.lines[..scroll.line.min(buffer.lines.len())]
        .iter()
        .map(|line| {
            line.layout_opt()
                .map_or(buffer.metrics().line_height, |layout| {
                    layout
                        .iter()
                        .map(|line| line.line_height_opt.unwrap_or(buffer.metrics().line_height))
                        .sum()
                })
        })
        .sum::<f32>()
        + scroll.vertical;
    (block_offset, scroll.horizontal)
}

fn draw_scrollbar(pixels: &mut [u8], width: u32, height: u32, scrollbar: Scrollbar) {
    let (track_start, track_length, track_x, track_y, track_width, track_height) =
        if scrollbar.horizontal {
            (
                0.0,
                width as f32,
                0.0,
                height as f32 - BAR_THICKNESS,
                width as f32,
                BAR_THICKNESS,
            )
        } else {
            (
                0.0,
                height as f32,
                width as f32 - BAR_THICKNESS,
                0.0,
                BAR_THICKNESS,
                height as f32,
            )
        };
    fill_rect(
        pixels,
        width,
        height,
        track_x,
        track_y,
        track_width,
        track_height,
        TRACK_COLOR,
    );

    let content_extent = scrollbar.content_extent.max(scrollbar.viewport_extent);
    let thumb_length = (track_length * scrollbar.viewport_extent / content_extent)
        .clamp(MIN_THUMB_LENGTH.min(track_length), track_length);
    let max_offset = (scrollbar.content_extent - scrollbar.viewport_extent).max(0.0);
    let thumb_offset = if max_offset > 0.0 {
        scrollbar.offset.clamp(0.0, max_offset) / max_offset * (track_length - thumb_length)
    } else {
        0.0
    };
    let (thumb_x, thumb_y, thumb_width, thumb_height) = if scrollbar.horizontal {
        (
            track_start + thumb_offset,
            track_y,
            thumb_length,
            BAR_THICKNESS,
        )
    } else {
        (
            track_x,
            track_start + thumb_offset,
            BAR_THICKNESS,
            thumb_length,
        )
    };
    fill_rect(
        pixels,
        width,
        height,
        thumb_x,
        thumb_y,
        thumb_width,
        thumb_height,
        THUMB_COLOR,
    );
}

fn draw_vertical_scrollbar(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    viewport: f32,
    content: f32,
    offset: f32,
) {
    let track_length = viewport.max(0.0);
    fill_rect(
        pixels,
        width,
        height,
        x,
        y,
        BAR_THICKNESS,
        track_length,
        TRACK_COLOR,
    );
    if track_length == 0.0 {
        return;
    }
    let content = content.max(viewport);
    let thumb_length =
        (track_length * viewport / content).clamp(MIN_THUMB_LENGTH.min(track_length), track_length);
    let max_offset = (content - viewport).max(0.0);
    let thumb_offset = if max_offset > 0.0 {
        offset.clamp(0.0, max_offset) / max_offset * (track_length - thumb_length)
    } else {
        0.0
    };
    fill_rect(
        pixels,
        width,
        height,
        x,
        y + thumb_offset,
        BAR_THICKNESS,
        thumb_length,
        THUMB_COLOR,
    );
}

fn draw_horizontal_scrollbar(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    viewport: f32,
    content: f32,
    offset: f32,
) {
    let track_length = viewport.max(0.0);
    fill_rect(
        pixels,
        width,
        height,
        x,
        y,
        track_length,
        BAR_THICKNESS,
        TRACK_COLOR,
    );
    if track_length == 0.0 {
        return;
    }
    let content = content.max(viewport);
    let thumb_length =
        (track_length * viewport / content).clamp(MIN_THUMB_LENGTH.min(track_length), track_length);
    let max_offset = (content - viewport).max(0.0);
    let thumb_offset = if max_offset > 0.0 {
        offset.clamp(0.0, max_offset) / max_offset * (track_length - thumb_length)
    } else {
        0.0
    };
    fill_rect(
        pixels,
        width,
        height,
        x + thumb_offset,
        y,
        thumb_length,
        BAR_THICKNESS,
        THUMB_COLOR,
    );
}

fn fill_rect(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    rect_width: f32,
    rect_height: f32,
    color: [u8; 4],
) {
    let left = x.floor().clamp(0.0, width as f32) as u32;
    let top = y.floor().clamp(0.0, height as f32) as u32;
    let right = (x + rect_width).ceil().clamp(0.0, width as f32) as u32;
    let bottom = (y + rect_height).ceil().clamp(0.0, height as f32) as u32;
    for row in top..bottom {
        for column in left..right {
            let index = ((row * width + column) * 4) as usize;
            pixels[index..index + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BAR_THICKNESS, Scrollbar, THUMB_COLOR, TRACK_COLOR, draw_scrollbar};

    #[test]
    fn thumb_tracks_scroll_offset() {
        let mut at_start = vec![255; 100 * 100 * 4];
        let mut scrolled = at_start.clone();
        let scrollbar = Scrollbar {
            horizontal: false,
            content_extent: 300.0,
            viewport_extent: 100.0,
            offset: 0.0,
            always_visible: true,
        };
        draw_scrollbar(&mut at_start, 100, 100, scrollbar);
        draw_scrollbar(
            &mut scrolled,
            100,
            100,
            Scrollbar {
                offset: 100.0,
                ..scrollbar
            },
        );
        assert_ne!(at_start, scrolled);
        let scrollbar_pixel = (100 - BAR_THICKNESS as usize) * 4;
        assert_eq!(
            &at_start[scrollbar_pixel..scrollbar_pixel + 4],
            &THUMB_COLOR
        );
        assert_eq!(
            &scrolled[scrollbar_pixel..scrollbar_pixel + 4],
            &TRACK_COLOR
        );
    }
}
