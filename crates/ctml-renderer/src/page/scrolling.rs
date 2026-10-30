use crate::style::{Overflow, WritingMode};

use super::{PageRasterizer, overflow_clip};

impl PageRasterizer {
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
            let (layout_width, layout_height) =
                super::vertical::layout_dimensions(self.writing_mode(), width, height);
            self.overflow_clips = overflow_clip::block_clip_rects(
                &self.runs,
                &self.buffer,
                layout_width,
                layout_height,
                self.writing_mode(),
            );
        }
        true
    }
}

pub(super) fn has_user_scrollable_axis(x: Overflow, y: Overflow) -> bool {
    let (x, y) = user_scrollable_axes(x, y);
    x || y
}

fn user_scrollable_axes(x: Overflow, y: Overflow) -> (bool, bool) {
    (is_user_scrollable(x, y), is_user_scrollable(y, x))
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

    use super::user_scrollable_axes;

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
}
