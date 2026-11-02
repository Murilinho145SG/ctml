use super::*;

impl PageRasterizer {
    pub(super) fn prepare_layout(&mut self, width: u32, height: u32) {
        self.layout_size = Some((width, height));
        self.flex_layout_offsets.retain(|(_, key), _| {
            !matches!(
                key,
                crate::page::FlexLayoutOffsetKey::PageLineX(_)
                    | crate::page::FlexLayoutOffsetKey::PageLineY(_)
            )
        });
        self.flex_item_box_x.clear();
        self.flex_column_extents.clear();
        for run in &mut self.runs {
            if run.style.flex_wrap_break {
                run.style.line_height = 1.0;
                run.style.flex_wrap_gap_applied = false;
            }
        }
        self.runs.retain(|run| !run.style.flow_spacer);
        let selection = self.selection.map(|(start, end)| {
            (
                cursor_without_flow_spacers(start, &self.spacer_line_flags),
                cursor_without_flow_spacers(end, &self.spacer_line_flags),
            )
        });
        let selection_anchor = self
            .selection_anchor
            .map(|cursor| cursor_without_flow_spacers(cursor, &self.spacer_line_flags));
        let viewport_size = viewport::resize_styles(self, width, height);
        let (layout_width, layout_height) =
            vertical::layout_dimensions(self.writing_mode(), width, height);
        let available_width = (layout_width as f32 - PAGE_PADDING * 2.0).max(1.0);
        let available_height = (layout_height as f32 - PAGE_PADDING * 2.0).max(1.0);
        let font_size = self.root_style.font_size;
        let root_intrinsic_widths = measure_root_intrinsic_widths(self);
        let vertical = self.writing_mode() != crate::style::WritingMode::HorizontalTb;
        let (
            inline_physical,
            inline_min_physical,
            inline_max_physical,
            inline_name,
            inline_min_name,
            inline_max_name,
        ) = if vertical {
            (
                &self.root_style.height,
                &self.root_style.min_height,
                &self.root_style.max_height,
                "height",
                "min-height",
                "max-height",
            )
        } else {
            (
                &self.root_style.width,
                &self.root_style.min_width,
                &self.root_style.max_width,
                "width",
                "min-width",
                "max-width",
            )
        };
        let inline_size = select_axis_size(
            self.root_style.inline_size.as_ref(),
            inline_physical.as_ref(),
            &self.root_style.size_property_orders,
            "inline-size",
            inline_name,
        )
        .flatten();
        let min_inline_size = select_axis_size(
            self.root_style.min_inline_size.as_ref(),
            inline_min_physical.as_ref(),
            &self.root_style.size_property_orders,
            "min-inline-size",
            inline_min_name,
        )
        .flatten();
        let max_inline_size = select_axis_size(
            self.root_style.max_inline_size.as_ref(),
            inline_max_physical.as_ref(),
            &self.root_style.size_property_orders,
            "max-inline-size",
            inline_max_name,
        )
        .flatten();
        let (
            block_physical,
            block_min_physical,
            block_max_physical,
            block_name,
            block_min_name,
            block_max_name,
        ) = if vertical {
            (
                &self.root_style.width,
                &self.root_style.min_width,
                &self.root_style.max_width,
                "width",
                "min-width",
                "max-width",
            )
        } else {
            (
                &self.root_style.height,
                &self.root_style.min_height,
                &self.root_style.max_height,
                "height",
                "min-height",
                "max-height",
            )
        };
        let block_size = select_axis_size(
            self.root_style.block_size.as_ref(),
            block_physical.as_ref(),
            &self.root_style.size_property_orders,
            "block-size",
            block_name,
        )
        .flatten();
        let min_block_size = select_axis_size(
            self.root_style.min_block_size.as_ref(),
            block_min_physical.as_ref(),
            &self.root_style.size_property_orders,
            "min-block-size",
            block_min_name,
        )
        .flatten();
        let max_block_size = select_axis_size(
            self.root_style.max_block_size.as_ref(),
            block_max_physical.as_ref(),
            &self.root_style.size_property_orders,
            "max-block-size",
            block_max_name,
        )
        .flatten();
        let content_width = resolve_intrinsic_size(
            inline_size,
            min_inline_size,
            max_inline_size,
            available_width,
            font_size,
            self.root_style.line_height,
            self.root_style.font_relative_units,
            viewport_size,
            root_intrinsic_widths,
        );
        let content_height = resolve_constrained_size(
            block_size,
            min_block_size,
            max_block_size,
            available_height,
            font_size,
            self.root_style.line_height,
            self.root_style.font_relative_units,
            viewport_size,
        );
        let page_writing_mode = self.writing_mode();
        layout::insert_first_letter_spacing(&mut self.runs, content_width, page_writing_mode);
        self.cursor_spans = cursor_spans(&self.runs);
        shape_buffer(self, content_width, content_height);
        let skipped_auto_nodes = if self
            .runs
            .iter()
            .any(|run| run.style.content_visibility_auto)
        {
            let mut auto_visibility_buffer = self.buffer.clone();
            auto_visibility_buffer.set_size(Some(content_width.max(content_height)), None);
            auto_visibility_buffer.set_scroll(Scroll::default());
            auto_visibility_buffer.shape_until_scroll(&mut self.font_system, false);
            let auto_visibility_geometries = overflow_clip::block_geometries(
                &self.runs,
                &mut auto_visibility_buffer,
                &mut self.font_system,
                layout_width,
                layout_height,
                page_writing_mode,
                &self.flex_layout_offsets,
                &self.flex_item_box_x,
                &self.flex_column_extents,
            );
            content_visibility::offscreen_auto_nodes(
                &self.runs,
                &auto_visibility_geometries,
                layout_width,
                layout_height,
                content_visibility::vertical_scroll_offset(&self.buffer),
                self.selection.is_some(),
                &self.focused_path,
            )
        } else {
            Default::default()
        };
        if !skipped_auto_nodes.is_empty() {
            content_visibility::skip_auto_contents(&mut self.runs, &skipped_auto_nodes);
            self.cursor_spans = cursor_spans(&self.runs);
        }
        let metrics = self.buffer.metrics();
        let line_sizes = line_layout_sizes(&self.runs, self.buffer.lines.len());
        let spacer_heights = block_spacer_heights(
            &self.runs,
            &line_sizes,
            &self.buffer,
            content_height,
            content_width,
            metrics.font_size,
            metrics.line_height,
            self.writing_mode(),
        );
        if let Some(runs) = insert_block_spacers(&self.runs, &spacer_heights) {
            self.runs = runs;
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        if layout::apply_first_line_word_spacing(&self.buffer, &mut self.runs, &self.cursor_spans) {
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        for _ in 0..8 {
            if !layout::apply_first_line_letter_spacing(
                &self.buffer,
                &mut self.runs,
                &self.cursor_spans,
            ) {
                break;
            }
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        if layout::insert_first_line_spacing(
            &self.buffer,
            &mut self.runs,
            &self.cursor_spans,
            content_width,
            page_writing_mode,
        ) {
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        if layout::apply_first_line_text_transform(&self.buffer, &mut self.runs, &self.cursor_spans)
        {
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        if layout::apply_first_line_line_height(&self.buffer, &mut self.runs, &self.cursor_spans) {
            self.cursor_spans = cursor_spans(&self.runs);
            shape_buffer(self, content_width, content_height);
        }
        self.spacer_line_flags = flow_spacer_lines(&self.runs, self.buffer.lines.len());
        self.selection = selection.map(|(start, end)| {
            (
                cursor_with_flow_spacers(start, &self.spacer_line_flags),
                cursor_with_flow_spacers(end, &self.spacer_line_flags),
            )
        });
        self.selection_anchor = selection_anchor
            .map(|cursor| cursor_with_flow_spacers(cursor, &self.spacer_line_flags));
        if !scrolling::has_user_scrollable_axis(
            self.root_style.overflow_x,
            self.root_style.overflow_y,
        ) {
            self.root_scroll = Scroll::default();
        }
        self.buffer.set_scroll(self.root_scroll);
        self.buffer.shape_until_scroll(&mut self.font_system, false);
        self.root_scroll = self.buffer.scroll();
        layout::apply_flex_columns(self, content_width, content_height);
        self.refresh_overflow_geometry(width, height);
    }

    pub(super) fn refresh_overflow_geometry(&mut self, width: u32, height: u32) {
        let (layout_width, layout_height) =
            vertical::layout_dimensions(self.writing_mode(), width, height);
        let writing_mode = self.writing_mode();
        let overflow_geometry = overflow_clip::block_geometries(
            &self.runs,
            &mut self.buffer,
            &mut self.font_system,
            layout_width,
            layout_height,
            writing_mode,
            &self.flex_layout_offsets,
            &self.flex_item_box_x,
            &self.flex_column_extents,
        );
        self.overflow_geometry = overflow_geometry;
        self.overflow_clips = self
            .overflow_geometry
            .iter()
            .filter(|(node_id, _)| {
                self.runs.iter().any(|run| {
                    run.style.layout_sizes.iter().any(|size| {
                        size.node_id == **node_id
                            && (size.overflow_x != crate::style::Overflow::Visible
                                || size.overflow_y != crate::style::Overflow::Visible
                                || size.paint_contained)
                    })
                })
            })
            .map(|(node_id, geometry)| (*node_id, geometry.clip))
            .collect();
        self.nested_scroll.retain(|node_id, _| {
            self.overflow_geometry.contains_key(node_id)
                && self.runs.iter().any(|run| {
                    run.style.layout_sizes.iter().any(|size| {
                        let (x_scrollable, y_scrollable) =
                            scrolling::user_scrollable_axes(size.overflow_x, size.overflow_y);
                        size.node_id == *node_id && (x_scrollable || y_scrollable)
                    })
                })
        });
        for (node_id, (offset_x, offset_y)) in &mut self.nested_scroll {
            if let Some(geometry) = self.overflow_geometry.get(node_id) {
                let (x_scrollable, y_scrollable) = self
                    .runs
                    .iter()
                    .flat_map(|run| &run.style.layout_sizes)
                    .find(|size| size.node_id == *node_id)
                    .map_or((false, false), |size| {
                        scrolling::user_scrollable_axes(size.overflow_x, size.overflow_y)
                    });
                let (max_x, max_y) = match writing_mode {
                    crate::style::WritingMode::HorizontalTb => (
                        (geometry.content_inline_extent
                            - (geometry.clip.right - geometry.clip.left))
                            .max(0.0),
                        (geometry.content_block_extent
                            - (geometry.clip.bottom - geometry.clip.top))
                            .max(0.0),
                    ),
                    crate::style::WritingMode::VerticalRl
                    | crate::style::WritingMode::VerticalLr => (
                        (geometry.content_block_extent
                            - (geometry.clip.bottom - geometry.clip.top))
                            .max(0.0),
                        (geometry.content_inline_extent
                            - (geometry.clip.right - geometry.clip.left))
                            .max(0.0),
                    ),
                };
                *offset_x = if x_scrollable {
                    offset_x.clamp(0.0, max_x)
                } else {
                    0.0
                };
                *offset_y = offset_y.clamp(0.0, if y_scrollable { max_y } else { 0.0 });
            }
        }
    }
}
