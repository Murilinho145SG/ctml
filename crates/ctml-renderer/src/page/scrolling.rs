use std::collections::HashSet;

use crate::style::{Overflow, WritingMode};

use super::PageRasterizer;

impl PageRasterizer {
    pub(crate) fn scroll_by_at(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        pointer_x: f64,
        pointer_y: f64,
    ) -> bool {
        let writing_mode = self.writing_mode();
        let (layout_x, layout_y) = self.layout_size.map_or((0.0, 0.0), |(width, height)| {
            super::vertical::layout_point(
                writing_mode,
                width,
                height,
                pointer_x as f32,
                pointer_y as f32,
            )
        });
        let mut remaining_x = delta_x;
        let mut remaining_y = delta_y;
        let mut nested_changed = false;
        for (node_id, max_x, max_y) in nested_scroll_targets(
            &self.runs,
            &self.overflow_geometry,
            layout_x,
            layout_y,
            writing_mode,
        ) {
            let current = self
                .nested_scroll
                .get(&node_id)
                .copied()
                .unwrap_or_default();
            let next = (
                (current.0 - remaining_x).clamp(0.0, max_x),
                (current.1 - remaining_y).clamp(0.0, max_y),
            );
            if next != current {
                self.nested_scroll.insert(node_id, next);
                remaining_x += next.0 - current.0;
                remaining_y += next.1 - current.1;
                nested_changed = true;
            }
        }
        let page_changed = self.scroll_by(remaining_x, remaining_y);
        nested_changed || page_changed
    }

    pub(crate) fn scroll_by(&mut self, delta_x: f32, delta_y: f32) -> bool {
        let (x_scrollable, y_scrollable) =
            user_scrollable_axes(self.root_style.overflow_x, self.root_style.overflow_y);
        if !x_scrollable && !y_scrollable {
            return false;
        }

        let mut scroll = self.buffer.scroll();
        match self.writing_mode() {
            WritingMode::HorizontalTb => {
                if x_scrollable {
                    scroll.horizontal -= delta_x;
                }
                if y_scrollable {
                    scroll.vertical -= delta_y;
                }
            }
            WritingMode::VerticalRl => {
                if x_scrollable {
                    scroll.vertical += delta_x;
                }
                if y_scrollable {
                    scroll.horizontal -= delta_y;
                }
            }
            WritingMode::VerticalLr => {
                if x_scrollable {
                    scroll.vertical -= delta_x;
                }
                if y_scrollable {
                    scroll.horizontal += delta_y;
                }
            }
        }
        let previous_scroll = self.buffer.scroll();
        if scroll == previous_scroll {
            return false;
        }

        self.buffer.set_scroll(scroll);
        self.buffer.shape_until_scroll(&mut self.font_system, false);
        self.root_scroll = self.buffer.scroll();
        if self.root_scroll == previous_scroll {
            return false;
        }
        if let Some((width, height)) = self.layout_size {
            self.refresh_overflow_geometry(width, height);
        }
        true
    }
}

pub(super) fn has_user_scrollable_axis(x: Overflow, y: Overflow) -> bool {
    let (x, y) = user_scrollable_axes(x, y);
    x || y
}

pub(super) fn user_scrollable_axes(x: Overflow, y: Overflow) -> (bool, bool) {
    (is_user_scrollable(x, y), is_user_scrollable(y, x))
}

fn nested_scroll_targets(
    runs: &[crate::style::TextRun],
    geometry: &std::collections::HashMap<usize, super::overflow_clip::BlockGeometry>,
    pointer_x: f32,
    pointer_y: f32,
    page_writing_mode: WritingMode,
) -> Vec<(usize, f32, f32)> {
    let mut candidates = HashSet::new();
    let mut targets = Vec::new();
    for run in runs {
        for size in &run.style.layout_sizes {
            if !candidates.insert(size.node_id) {
                continue;
            }
            let (x_scrollable, y_scrollable) =
                user_scrollable_axes(size.overflow_x, size.overflow_y);
            if (!x_scrollable && !y_scrollable) || size.writing_mode != page_writing_mode {
                continue;
            }
            let Some(block) = geometry.get(&size.node_id) else {
                continue;
            };
            let clip = block.clip;
            if pointer_x < clip.left
                || pointer_x >= clip.right
                || pointer_y < clip.top
                || pointer_y >= clip.bottom
            {
                continue;
            }
            let (inline_viewport, block_viewport) =
                (clip.right - clip.left, clip.bottom - clip.top);
            let (content_x, viewport_x, content_y, viewport_y) = match page_writing_mode {
                WritingMode::HorizontalTb => (
                    block.content_inline_extent,
                    inline_viewport,
                    block.content_block_extent,
                    block_viewport,
                ),
                WritingMode::VerticalRl | WritingMode::VerticalLr => (
                    block.content_block_extent,
                    block_viewport,
                    block.content_inline_extent,
                    inline_viewport,
                ),
            };
            let max_x = if x_scrollable {
                (content_x - viewport_x).max(0.0)
            } else {
                0.0
            };
            let max_y = if y_scrollable {
                (content_y - viewport_y).max(0.0)
            } else {
                0.0
            };
            if max_x <= 0.0 && max_y <= 0.0 {
                continue;
            }
            let depth = run
                .element_path
                .iter()
                .position(|node_id| *node_id == size.node_id)
                .unwrap_or_default();
            targets.push((size.node_id, max_x, max_y, depth));
        }
    }
    targets.sort_by_key(|(_, _, _, depth)| std::cmp::Reverse(*depth));
    targets
        .into_iter()
        .map(|(node_id, max_x, max_y, _)| (node_id, max_x, max_y))
        .collect()
}

pub(super) fn scrollbar_axis(value: Overflow, other: Overflow) -> Option<bool> {
    match value {
        Overflow::Auto => Some(false),
        Overflow::Scroll => Some(true),
        Overflow::Visible if !matches!(other, Overflow::Visible | Overflow::Clip) => Some(false),
        Overflow::Visible | Overflow::Hidden | Overflow::Clip => None,
    }
}

fn is_user_scrollable(value: Overflow, other: Overflow) -> bool {
    match value {
        Overflow::Auto | Overflow::Scroll => true,
        Overflow::Visible => !matches!(other, Overflow::Visible | Overflow::Clip),
        Overflow::Hidden | Overflow::Clip => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::style::Overflow;

    use super::{scrollbar_axis, user_scrollable_axes};

    #[test]
    fn computes_user_scrollable_overflow_axes() {
        assert_eq!(
            user_scrollable_axes(Overflow::Auto, Overflow::Visible),
            (true, true)
        );
        assert_eq!(
            user_scrollable_axes(Overflow::Scroll, Overflow::Clip),
            (true, false)
        );
        assert_eq!(
            user_scrollable_axes(Overflow::Hidden, Overflow::Visible),
            (false, true)
        );
        assert_eq!(
            user_scrollable_axes(Overflow::Visible, Overflow::Clip),
            (false, false)
        );
    }

    #[test]
    fn computes_scrollbar_modes_for_visible_and_scroll_axes() {
        assert_eq!(scrollbar_axis(Overflow::Scroll, Overflow::Clip), Some(true));
        assert_eq!(
            scrollbar_axis(Overflow::Auto, Overflow::Visible),
            Some(false)
        );
        assert_eq!(
            scrollbar_axis(Overflow::Visible, Overflow::Hidden),
            Some(false)
        );
        assert_eq!(scrollbar_axis(Overflow::Visible, Overflow::Clip), None);
        assert_eq!(scrollbar_axis(Overflow::Hidden, Overflow::Visible), None);
    }
}
