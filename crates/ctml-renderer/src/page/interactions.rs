use cosmic_text::{Affinity, Cursor};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use winit::window::CursorIcon;

use super::{PAGE_PADDING, PageRasterizer, local_writing};

impl PageRasterizer {
    pub(crate) fn cursor_at(&self, x: f64, y: f64) -> (CursorIcon, bool) {
        let fallback = if self.root_style.cursor == CursorIcon::Text {
            CursorIcon::Default
        } else {
            self.root_style.cursor
        };
        let fallback_visible = !self.root_style.cursor_hidden;
        let Some(cursor) = self.hit(x, y) else {
            return (fallback, fallback_visible);
        };
        self.cursor_spans
            .iter()
            .find(|span| {
                span.line == cursor.line
                    && cursor.index >= span.start
                    && cursor.index < span.end
                    && !span.visibility_hidden
                    && !span.pointer_events_none
            })
            .or_else(|| {
                self.cursor_spans.iter().rev().find(|span| {
                    span.line == cursor.line
                        && cursor.index == span.end
                        && !span.visibility_hidden
                        && !span.pointer_events_none
                })
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
        let Some(cursor) = self.selection_hit(x, y) else {
            let changed = self.selection.take().is_some();
            self.selection_anchor = None;
            return changed || active_changed || focus_changed || modality_changed;
        };
        self.selection_anchor = Some(cursor);
        self.selection = self
            .user_select_owner_at(cursor)
            .and_then(|owner| self.user_select_owner_range(owner))
            .or(Some((cursor, cursor)));
        true
    }

    pub(crate) fn extend_selection(&mut self, x: f64, y: f64) -> bool {
        let Some(anchor) = self.selection_anchor else {
            return false;
        };
        let Some(cursor) = self.selection_hit(x, y) else {
            return false;
        };
        let selection = self
            .user_select_owner_at(anchor)
            .and_then(|owner| self.user_select_owner_range(owner))
            .map_or((anchor, cursor), |(owner_start, owner_end)| {
                if cursor < owner_start {
                    (cursor, owner_end)
                } else if cursor > owner_end {
                    (owner_start, cursor)
                } else {
                    (owner_start, owner_end)
                }
            });
        if self.selection == Some(selection) {
            return false;
        }
        self.selection = Some(selection);
        true
    }

    pub(crate) fn select_word_at(&mut self, x: f64, y: f64) -> bool {
        let Some(cursor) = self.selection_hit(x, y) else {
            let changed = self.selection.take().is_some();
            self.selection_anchor = None;
            return changed;
        };
        if let Some(selection) = self
            .user_select_owner_at(cursor)
            .and_then(|owner| self.user_select_owner_range(owner))
        {
            self.selection = Some(selection);
            self.selection_anchor = None;
            return true;
        }
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
                span.line == cursor.line
                    && cursor.index >= span.start
                    && cursor.index < span.end
                    && !span.visibility_hidden
                    && !span.pointer_events_none
            })
            .or_else(|| {
                self.cursor_spans.iter().rev().find(|span| {
                    span.line == cursor.line
                        && cursor.index == span.end
                        && !span.visibility_hidden
                        && !span.pointer_events_none
                })
            })
            .map_or_else(Vec::new, |span| span.element_path.clone())
    }

    fn hit(&self, x: f64, y: f64) -> Option<Cursor> {
        let (x, y) = self
            .local_vertical_layout_point(x as f32, y as f32)
            .unwrap_or_else(|| self.layout_point(x as f32, y as f32));
        let x = x - PAGE_PADDING;
        let y = y - PAGE_PADDING;
        let inside_text_line = self.buffer.layout_runs().any(|run| {
            run.line_w > 0.0
                && x >= 0.0
                && x <= run.line_w
                && y >= run.line_top
                && y < run.line_top + run.line_height
        });
        inside_text_line
            .then(|| self.buffer.hit(x, y))
            .flatten()
            .filter(|cursor| {
                self.cursor_spans.iter().any(|span| {
                    span.line == cursor.line
                        && cursor.index >= span.start
                        && cursor.index < span.end
                        && !span.visibility_hidden
                        && !span.pointer_events_none
                })
            })
    }

    fn selection_hit(&self, x: f64, y: f64) -> Option<Cursor> {
        self.hit(x, y).filter(|cursor| {
            self.cursor_spans.iter().any(|span| {
                span.line == cursor.line
                    && cursor.index >= span.start
                    && cursor.index < span.end
                    && !span.user_select_none
            })
        })
    }

    fn user_select_owner_at(&self, cursor: Cursor) -> Option<usize> {
        self.cursor_spans
            .iter()
            .find(|span| {
                span.line == cursor.line && cursor.index >= span.start && cursor.index < span.end
            })
            .and_then(|span| span.user_select_all_owner)
    }

    fn user_select_owner_range(&self, owner: usize) -> Option<(Cursor, Cursor)> {
        let mut spans = self.cursor_spans.iter().filter(|span| {
            span.user_select_all_owner == Some(owner)
                && !span.visibility_hidden
                && !span.user_select_none
        });
        let first = spans.next()?;
        let mut last = first;
        for span in spans {
            last = span;
        }
        Some((
            Cursor::new_with_affinity(first.line, first.start, Affinity::Before),
            Cursor::new_with_affinity(last.line, last.end, Affinity::After),
        ))
    }

    fn layout_point(&self, x: f32, y: f32) -> (f32, f32) {
        let (width, height) = self.layout_size.unwrap_or((0, 0));
        super::vertical::layout_point(self.writing_mode(), width, height, x, y)
    }

    fn local_vertical_layout_point(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.writing_mode() != crate::style::WritingMode::HorizontalTb {
            return None;
        }
        let (width, height) = self.layout_size?;
        let style_ranges = local_writing::text_style_ranges(&self.runs);
        let line_starts = local_writing::text_line_starts(&self.runs);
        let origins =
            local_writing::vertical_block_origins(&self.runs, &self.buffer, self.writing_mode());
        let mut bounds = HashMap::<usize, (crate::style::WritingMode, f32, f32, f32, f32)>::new();
        for line in self.buffer.layout_runs() {
            let line_start = line_starts.get(line.line_i).copied().unwrap_or(0);
            for glyph in line.glyphs {
                let Some(style_index) =
                    local_writing::style_index_at(&style_ranges, line_start + glyph.start)
                else {
                    continue;
                };
                let Some(run) = self.runs.get(style_index) else {
                    continue;
                };
                if run.style.writing_mode == crate::style::WritingMode::HorizontalTb {
                    continue;
                }
                let Some(block) = run.first_line_block else {
                    continue;
                };
                let entry = bounds.entry(block).or_insert((
                    run.style.writing_mode,
                    glyph.x,
                    glyph.x + glyph.w,
                    line.line_top,
                    line.line_top + line.line_height,
                ));
                entry.1 = entry.1.min(glyph.x);
                entry.2 = entry.2.max(glyph.x + glyph.w);
                entry.3 = entry.3.min(line.line_top);
                entry.4 = entry.4.max(line.line_top + line.line_height);
            }
        }
        for (block, (mode, left, right, top, bottom)) in bounds {
            let Some(origin_y) = origins.get(&block).copied() else {
                continue;
            };
            let transform =
                local_writing::LocalVerticalTransform::new(mode, origin_y, width, height);
            let (rect_left, rect_top, rect_width, rect_height) = transform.map_rect(
                left + PAGE_PADDING,
                top + PAGE_PADDING,
                right - left,
                bottom - top,
            );
            if x >= rect_left
                && x <= rect_left + rect_width
                && y >= rect_top
                && y <= rect_top + rect_height
            {
                let (layout_x, layout_y) = super::vertical::layout_point(mode, width, height, x, y);
                return Some((layout_x, layout_y));
            }
        }
        None
    }
}
