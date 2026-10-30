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
        if self.writing_mode() == WritingMode::HorizontalTb {
            let (layout_x, layout_y) = self.layout_size.map_or((0.0, 0.0), |(width, height)| {
                super::vertical::layout_point(
                    self.writing_mode(),
                    width,
                    height,
                    pointer_x as f32,
                    pointer_y as f32,
                )
            });
            if let Some((node_id, max_x, max_y)) =
                nested_scroll_target(&self.runs, &self.overflow_geometry, layout_x, layout_y)
            {
                let current = self
                    .nested_scroll
                    .get(&node_id)
                    .copied()
                    .unwrap_or_default();
                let next = (
                    (current.0 - delta_x).clamp(0.0, max_x),
                    (current.1 - delta_y).clamp(0.0, max_y),
                );
                if next != current {
                    self.nested_scroll.insert(node_id, next);
                    return true;
                }
            }
        }
        self.scroll_by(delta_x, delta_y)
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

fn nested_scroll_target(
    runs: &[crate::style::TextRun],
    geometry: &std::collections::HashMap<usize, super::overflow_clip::BlockGeometry>,
    pointer_x: f32,
    pointer_y: f32,
) -> Option<(usize, f32, f32)> {
    let mut candidates = HashSet::new();
    let mut target: Option<(usize, f32, f32, usize)> = None;
    for run in runs {
        for size in &run.style.layout_sizes {
            if !candidates.insert(size.node_id) {
                continue;
            }
            let (x_scrollable, y_scrollable) =
                user_scrollable_axes(size.overflow_x, size.overflow_y);
            if (!x_scrollable && !y_scrollable) || size.writing_mode != WritingMode::HorizontalTb {
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
            let max_x = if x_scrollable {
                (block.content_inline_extent - (clip.right - clip.left)).max(0.0)
            } else {
                0.0
            };
            let max_y = if y_scrollable {
                (block.content_block_extent - (clip.bottom - clip.top)).max(0.0)
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
            if target.is_none_or(|(_, _, _, current_depth)| depth >= current_depth) {
                target = Some((size.node_id, max_x, max_y, depth));
            }
        }
    }
    target.map(|(node_id, max_x, max_y, _)| (node_id, max_x, max_y))
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
