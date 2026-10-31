use super::super::{FlexAlignment, FlexBasis, TextRun, TextStyle, append_text};
use super::{append_gap, explicit_row_cross_size};

#[derive(Default)]
pub(in crate::style) struct FlexGapState {
    item_count: usize,
    last_item_id: Option<usize>,
    last_item_grow: f32,
    last_item_shrink: f32,
    last_item_basis: Option<FlexBasis>,
    last_item_min_main_size: Option<String>,
    last_item_max_main_size: Option<String>,
}

pub(in crate::style) fn item_alignment(
    style: &TextStyle,
    inherited: &TextStyle,
    block: bool,
) -> Option<FlexAlignment> {
    if style.display_flex && block {
        if style.flex_direction_column {
            Some(style.align_items.unwrap_or(FlexAlignment::Stretch))
        } else {
            Some(visual_justify(
                style.justify_content,
                style.flex_direction_reverse,
            ))
        }
    } else if inherited.flex_items_row {
        Some(visual_justify(
            inherited.justify_content,
            inherited.flex_direction_reverse,
        ))
    } else if inherited.flex_items_column {
        Some(
            style
                .align_self
                .or(inherited.align_items)
                .unwrap_or(FlexAlignment::Stretch),
        )
    } else {
        inherited.flex_item_alignment
    }
}

fn visual_justify(alignment: FlexAlignment, reverse: bool) -> FlexAlignment {
    if !reverse {
        return alignment;
    }
    match alignment {
        FlexAlignment::FlexStart => FlexAlignment::FlexEnd,
        FlexAlignment::FlexEnd => FlexAlignment::FlexStart,
        FlexAlignment::Start
        | FlexAlignment::End
        | FlexAlignment::Center
        | FlexAlignment::Stretch
        | FlexAlignment::SpaceBetween
        | FlexAlignment::SpaceAround
        | FlexAlignment::SpaceEvenly => alignment,
    }
}

impl FlexGapState {
    pub(in crate::style) fn append_text(
        &mut self,
        style: &TextStyle,
        item_id: usize,
        text: &str,
        element_path: &[usize],
        output: &mut Vec<TextRun>,
        last_was_space: &mut bool,
        capitalize_next: &mut bool,
    ) {
        self.before_item(
            style,
            item_id,
            !text.chars().all(char::is_whitespace),
            0.0,
            1.0,
            None,
            None,
            None,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
        append_text(
            text,
            style,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
    }

    pub(in crate::style) fn before_element(
        &mut self,
        style: &TextStyle,
        item_id: usize,
        grow: f32,
        shrink: f32,
        basis: Option<FlexBasis>,
        min_main_size: Option<String>,
        max_main_size: Option<String>,
        element_path: &[usize],
        output: &mut Vec<TextRun>,
        last_was_space: &mut bool,
        capitalize_next: &mut bool,
    ) {
        self.before_item(
            style,
            item_id,
            true,
            grow,
            shrink,
            basis,
            min_main_size,
            max_main_size,
            element_path,
            output,
            last_was_space,
            capitalize_next,
        );
    }

    fn before_item(
        &mut self,
        style: &TextStyle,
        item_id: usize,
        is_item: bool,
        grow: f32,
        shrink: f32,
        basis: Option<FlexBasis>,
        min_main_size: Option<String>,
        max_main_size: Option<String>,
        element_path: &[usize],
        output: &mut Vec<TextRun>,
        last_was_space: &mut bool,
        capitalize_next: &mut bool,
    ) {
        if !is_item || !(style.flex_items_row || style.flex_items_column) {
            return;
        }
        if self.last_item_id == Some(item_id) {
            return;
        }
        if self.item_count == 0
            && style.flex_items_row
            && matches!(
                style.justify_content,
                FlexAlignment::SpaceAround | FlexAlignment::SpaceEvenly
            )
        {
            append_gap(
                style,
                true,
                0.0,
                0.0,
                None,
                None,
                None,
                true,
                element_path,
                output,
                last_was_space,
                capitalize_next,
            );
        }
        if self.item_count > 0 {
            append_gap(
                style,
                false,
                self.last_item_grow,
                self.last_item_shrink,
                self.last_item_basis.clone(),
                self.last_item_min_main_size.clone(),
                self.last_item_max_main_size.clone(),
                is_justify_distribution(style),
                element_path,
                output,
                last_was_space,
                capitalize_next,
            );
        }
        self.item_count += 1;
        self.last_item_id = Some(item_id);
        self.last_item_grow = grow;
        self.last_item_shrink = shrink;
        self.last_item_basis = basis;
        self.last_item_min_main_size = min_main_size;
        self.last_item_max_main_size = max_main_size;
    }

    pub(in crate::style) fn finish(
        &self,
        style: &TextStyle,
        element_path: &[usize],
        output: &mut Vec<TextRun>,
        last_was_space: &mut bool,
        capitalize_next: &mut bool,
    ) {
        if self.item_count == 0 || !style.flex_items_row {
            return;
        }
        let trailing_justify = style.justify_content == FlexAlignment::SpaceEvenly;
        if trailing_justify
            || self.last_item_grow > 0.0
            || self.last_item_shrink > 0.0
            || self.last_item_basis.is_some()
            || self.last_item_min_main_size.is_some()
            || self.last_item_max_main_size.is_some()
            || (style.flex_wrap
                && style.align_content.unwrap_or(FlexAlignment::Stretch) == FlexAlignment::Stretch
                && explicit_row_cross_size(style).is_some())
        {
            append_gap(
                style,
                false,
                self.last_item_grow,
                self.last_item_shrink,
                self.last_item_basis.clone(),
                self.last_item_min_main_size.clone(),
                self.last_item_max_main_size.clone(),
                trailing_justify,
                element_path,
                output,
                last_was_space,
                capitalize_next,
            );
        }
    }
}

fn is_justify_distribution(style: &TextStyle) -> bool {
    matches!(
        style.justify_content,
        FlexAlignment::SpaceBetween | FlexAlignment::SpaceAround | FlexAlignment::SpaceEvenly
    )
}
