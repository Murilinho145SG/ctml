use cosmic_text::{Affinity, Cursor};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use winit::window::CursorIcon;

use super::{PageRasterizer, local_writing, overflow_clip, page_padding};

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
        self.span_at_cursor(cursor)
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
        if self.has_interaction_rules {
            self.rebuild_interaction_styles();
            true
        } else {
            false
        }
    }

    pub(crate) fn clear_hover(&mut self) -> bool {
        if self.hovered_path.is_empty() {
            return false;
        }
        self.hovered_path.clear();
        if self.has_interaction_rules {
            self.rebuild_interaction_styles();
            true
        } else {
            false
        }
    }

    pub(crate) fn focus_next(&mut self, backwards: bool) -> bool {
        let Some(focused_path) = self.focus_manager.next(&self.focused_path, backwards) else {
            return false;
        };
        let changed = self.focused_path != focused_path || !self.keyboard_focus;
        if changed {
            self.focused_path = focused_path;
            self.keyboard_focus = true;
            if self.has_interaction_rules {
                self.rebuild_interaction_styles();
            }
        }
        changed && self.has_interaction_rules
    }

    pub(crate) fn action_at(&self, x: f64, y: f64) -> Option<String> {
        let path = self.element_path_at(x, y);
        self.focus_manager.action_for_path(&path).map(str::to_owned)
    }

    pub(crate) fn focused_action(&self) -> Option<String> {
        self.focus_manager
            .action_for_focus(&self.focused_path)
            .map(str::to_owned)
    }

    pub(crate) fn focused_text_input(&self) -> Option<(Vec<usize>, String, String)> {
        self.focus_manager
            .text_input_for_focus(&self.focused_path)
            .map(|input| (input.path.clone(), input.id.clone(), input.value.clone()))
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
        if self.has_interaction_rules && (active_changed || focus_changed || modality_changed) {
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
        let Some(cursor) = self.selection_hit_with_clamp(x, y, true) else {
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
        if self.has_interaction_rules {
            self.rebuild_interaction_styles();
            true
        } else {
            false
        }
    }

    fn element_path_at(&self, x: f64, y: f64) -> Vec<usize> {
        let Some(cursor) = self.hit(x, y) else {
            return Vec::new();
        };
        self.span_at_cursor(cursor)
            .map_or_else(Vec::new, |span| span.element_path.clone())
    }

    fn hit(&self, x: f64, y: f64) -> Option<Cursor> {
        self.hit_with_clamp(x, y, false)
    }

    fn hit_with_clamp(&self, x: f64, y: f64, clamp: bool) -> Option<Cursor> {
        let (pointer_x, pointer_y) = self.layout_point(x as f32, y as f32);
        let (x, y) = self
            .local_vertical_layout_point(x as f32, y as f32)
            .unwrap_or((pointer_x, pointer_y));
        let x = x - page_padding();
        let (scroll_offset_x, scroll_offset_y) = self
            .overflow_geometry
            .iter()
            .filter(|(node_id, geometry)| {
                self.nested_scroll.contains_key(node_id)
                    && pointer_x >= geometry.clip.left
                    && pointer_x < geometry.clip.right
                    && pointer_y >= geometry.clip.top
                    && pointer_y < geometry.clip.bottom
            })
            .filter_map(|(node_id, _)| self.nested_scroll.get(node_id).copied())
            .fold((0.0, 0.0), |(offset_x, offset_y), (x, y)| {
                (offset_x + x, offset_y + y)
            });
        let (scroll_translation_x, scroll_translation_y) = match self.writing_mode() {
            crate::style::WritingMode::HorizontalTb => (scroll_offset_x, scroll_offset_y),
            crate::style::WritingMode::VerticalRl => (-scroll_offset_y, scroll_offset_x),
            crate::style::WritingMode::VerticalLr => (scroll_offset_y, -scroll_offset_x),
        };
        let x = x + scroll_translation_x;
        let y = y - page_padding() + scroll_translation_y;
        let mut x = x;
        let mut y = y;
        let hit_line = if clamp {
            let line = self
                .layout_runs()
                .filter(|run| run.line_w > 0.0)
                .min_by(|left, right| {
                    let distance = |run: &super::PageLayoutRun<'_>| {
                        (run.line_top - y).max(0.0) + (y - run.line_top - run.line_height).max(0.0)
                    };
                    distance(left).total_cmp(&distance(right))
                });
            if let Some(line) = &line {
                let left = line
                    .glyphs
                    .iter()
                    .map(|glyph| glyph.x)
                    .fold(f32::INFINITY, f32::min);
                let right = line
                    .glyphs
                    .iter()
                    .map(|glyph| glyph.x + glyph.w)
                    .fold(f32::NEG_INFINITY, f32::max);
                x = x.clamp(left, right);
                y = y.clamp(line.line_top, line.line_top + line.line_height * 0.999);
            }
            line
        } else {
            // line_w is the text advance, excluding CSS indentation and alignment offsets.
            self.layout_runs().find(|run| {
                run.line_w > 0.0
                    && y >= run.line_top
                    && y < run.line_top + run.line_height
                    && run
                        .glyphs
                        .iter()
                        .any(|glyph| x >= glyph.x && x <= glyph.x + glyph.w)
            })
        };
        let hit_offset = hit_line
            .as_ref()
            .map(|run| {
                let line_offset = run
                    .glyphs
                    .first()
                    .and_then(|glyph| {
                        self.flex_layout_offsets
                            .get(&(run.line_i, super::FlexLayoutOffsetKey::Line(glyph.start)))
                    })
                    .copied()
                    .unwrap_or_default();
                let glyph_offset = run
                    .glyphs
                    .iter()
                    .find(|glyph| x >= glyph.x && x <= glyph.x + glyph.w)
                    .and_then(|glyph| {
                        self.flex_layout_offsets
                            .get(&(run.line_i, super::FlexLayoutOffsetKey::Glyph(glyph.start)))
                    })
                    .copied()
                    .unwrap_or_default();
                line_offset + glyph_offset
            })
            .unwrap_or_default();
        let cursor = hit_line.and_then(|_| self.buffer.hit(x, y - hit_offset))?;
        let span = self.span_at_cursor(cursor)?;
        if span.visibility_hidden || span.pointer_events_none {
            return None;
        }
        overflow_clip::clip_for_path(&span.element_path, &self.overflow_clips)
            .is_none_or(|clip| {
                pointer_x >= clip.left
                    && pointer_x < clip.right
                    && pointer_y >= clip.top
                    && pointer_y < clip.bottom
            })
            .then_some(cursor)
    }

    fn span_at_cursor(&self, cursor: Cursor) -> Option<&super::CursorSpan> {
        self.cursor_spans.iter().find(|span| {
            span.line == cursor.line
                && cursor.index >= span.start
                && (cursor.index < span.end
                    || (cursor.index == span.end && cursor.affinity == Affinity::Before))
        })
    }

    fn selection_hit(&self, x: f64, y: f64) -> Option<Cursor> {
        self.selection_hit_with_clamp(x, y, false)
    }

    fn selection_hit_with_clamp(&self, x: f64, y: f64, clamp: bool) -> Option<Cursor> {
        self.hit_with_clamp(x, y, clamp).filter(|cursor| {
            self.span_at_cursor(*cursor)
                .is_some_and(|span| !span.user_select_none)
        })
    }

    fn user_select_owner_at(&self, cursor: Cursor) -> Option<usize> {
        self.span_at_cursor(cursor)
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
        if self.writing_mode() != crate::style::WritingMode::HorizontalTb
            || self
                .runs
                .iter()
                .all(|run| run.style.writing_mode == crate::style::WritingMode::HorizontalTb)
        {
            return None;
        }
        let (width, height) = self.layout_size?;
        let style_ranges = local_writing::text_style_ranges(&self.runs);
        let line_starts = local_writing::text_line_starts(&self.runs);
        let origins = local_writing::vertical_block_origins(
            &self.runs,
            &self.buffer,
            self.writing_mode(),
            &self.flex_layout_offsets,
        );
        let mut bounds = HashMap::<usize, (crate::style::WritingMode, f32, f32, f32, f32)>::new();
        for line in self.layout_runs() {
            let line_start = line_starts.get(line.line_i).copied().unwrap_or(0);
            for glyph in line.glyphs.iter() {
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
                left + page_padding(),
                top + page_padding(),
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
