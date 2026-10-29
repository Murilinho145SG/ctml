use cosmic_text::{Affinity, Cursor};
use unicode_segmentation::UnicodeSegmentation;
use winit::window::CursorIcon;

use super::{PAGE_PADDING, PageRasterizer};

impl PageRasterizer {
    pub(crate) fn cursor_at(&self, x: f64, y: f64) -> (CursorIcon, bool) {
        let fallback = if self.root_style.cursor == CursorIcon::Text {
            CursorIcon::Default
        } else {
            self.root_style.cursor
        };
        let fallback_visible = !self.root_style.cursor_hidden;
        if x < PAGE_PADDING as f64 || y < PAGE_PADDING as f64 {
            return (fallback, fallback_visible);
        }
        let Some(cursor) = self.hit(x, y) else {
            return (fallback, fallback_visible);
        };
        self.cursor_spans
            .iter()
            .find(|span| {
                span.line == cursor.line && cursor.index >= span.start && cursor.index < span.end
            })
            .or_else(|| {
                self.cursor_spans
                    .iter()
                    .rev()
                    .find(|span| span.line == cursor.line && cursor.index == span.end)
            })
            .map_or((fallback, fallback_visible), |span| {
                (span.icon, !span.hidden)
            })
    }

    pub(crate) fn update_hover(&mut self, x: f64, y: f64) -> bool {
        let path = self.element_path_at(x, y);
        if path == self.hovered_path {
            return false;
        }
        self.hovered_path = path;
        self.rebuild_interaction_styles();
        true
    }

    pub(crate) fn clear_hover(&mut self) -> bool {
        if self.hovered_path.is_empty() {
            return false;
        }
        self.hovered_path.clear();
        self.rebuild_interaction_styles();
        true
    }

    pub(crate) fn focus_next(&mut self, backwards: bool) -> bool {
        let Some(focused_path) = self.focus_manager.next(&self.focused_path, backwards) else {
            return false;
        };
        let changed = self.focused_path != focused_path || !self.keyboard_focus;
        if changed {
            self.focused_path = focused_path;
            self.keyboard_focus = true;
            self.rebuild_interaction_styles();
        }
        changed
    }

    pub(crate) fn begin_selection(&mut self, x: f64, y: f64) -> bool {
        let path = self.element_path_at(x, y);
        let focused_path = path
            .iter()
            .rposition(|node_id| self.focus_manager.contains(node_id))
            .map_or_else(Vec::new, |index| path[..=index].to_vec());
        let focus_changed = self.focused_path != focused_path;
        if focus_changed {
            self.focused_path = focused_path;
        }
        let modality_changed = self.keyboard_focus;
        self.keyboard_focus = false;
        let active_changed = path != self.active_path;
        if active_changed {
            self.active_path = path.clone();
        }
        if active_changed || focus_changed || modality_changed {
            self.rebuild_interaction_styles();
        }
        let Some(cursor) = self.hit(x, y) else {
            let changed = self.selection.take().is_some();
            self.selection_anchor = None;
            return changed || active_changed || focus_changed || modality_changed;
        };
        self.selection_anchor = Some(cursor);
        self.selection = Some((cursor, cursor));
        true
    }

    pub(crate) fn extend_selection(&mut self, x: f64, y: f64) -> bool {
        let Some(anchor) = self.selection_anchor else {
            return false;
        };
        let Some(cursor) = self.hit(x, y) else {
            return false;
        };
        let selection = (anchor, cursor);
        if self.selection == Some(selection) {
            return false;
        }
        self.selection = Some(selection);
        true
    }

    pub(crate) fn select_word_at(&mut self, x: f64, y: f64) -> bool {
        let Some(cursor) = self.hit(x, y) else {
            let changed = self.selection.take().is_some();
            self.selection_anchor = None;
            return changed;
        };
        let Some(text) = self
            .buffer
            .layout_runs()
            .find(|run| run.line_i == cursor.line)
            .map(|run| run.text.to_owned())
        else {
            return false;
        };
        let target = match cursor.affinity {
            Affinity::After => cursor.index,
            Affinity::Before => text[..cursor.index]
                .char_indices()
                .last()
                .map_or(cursor.index, |(index, _)| index),
        };
        let Some((start, segment)) = text
            .split_word_bound_indices()
            .find(|(start, segment)| target >= *start && target < *start + segment.len())
        else {
            return false;
        };
        let start = Cursor::new_with_affinity(cursor.line, start, Affinity::After);
        let end =
            Cursor::new_with_affinity(cursor.line, start.index + segment.len(), Affinity::Before);
        self.selection = Some((start, end));
        self.selection_anchor = None;
        true
    }

    pub(crate) fn end_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub(crate) fn end_active(&mut self) -> bool {
        if self.active_path.is_empty() {
            return false;
        }
        self.active_path.clear();
        self.rebuild_interaction_styles();
        true
    }

    fn element_path_at(&self, x: f64, y: f64) -> Vec<usize> {
        let Some(cursor) = self.hit(x, y) else {
            return Vec::new();
        };
        self.cursor_spans
            .iter()
            .find(|span| {
                span.line == cursor.line && cursor.index >= span.start && cursor.index < span.end
            })
            .or_else(|| {
                self.cursor_spans
                    .iter()
                    .rev()
                    .find(|span| span.line == cursor.line && cursor.index == span.end)
            })
            .map_or_else(Vec::new, |span| span.element_path.clone())
    }

    fn hit(&self, x: f64, y: f64) -> Option<Cursor> {
        let x = x as f32 - PAGE_PADDING;
        let y = y as f32 - PAGE_PADDING;
        let inside_text_line = self.buffer.layout_runs().any(|run| {
            run.line_w > 0.0
                && x >= 0.0
                && x <= run.line_w
                && y >= run.line_top
                && y < run.line_top + run.line_height
        });
        inside_text_line.then(|| self.buffer.hit(x, y)).flatten()
    }
}
