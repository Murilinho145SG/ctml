use crate::style::{FlexAlignment, TextStyle};

pub(super) fn apply_place_content(style: &mut TextStyle, value: &str) {
    let values = value.split_ascii_whitespace().collect::<Vec<_>>();
    if values.is_empty() || values.len() > 2 {
        return;
    }
    let align_content = values[0].to_ascii_lowercase();
    let mut justify_content = values
        .get(1)
        .copied()
        .unwrap_or(values[0])
        .to_ascii_lowercase();
    if !is_supported_align_content(&align_content)
        || !is_supported_justify_content(&justify_content)
    {
        return;
    }
    if justify_content == "stretch" {
        justify_content = "flex-start".to_owned();
    }
    super::display::apply_align_content(style, &align_content);
    super::display::apply_justify_content(style, &justify_content);
}

pub(super) fn reset_place_content(style: &mut TextStyle) {
    style.align_content = None;
    style.justify_content = FlexAlignment::default();
}

pub(super) fn inherit_place_content(style: &mut TextStyle, inherited: &TextStyle) {
    style.align_content = inherited.align_content;
    style.justify_content = inherited.justify_content;
}

fn is_supported_align_content(value: &str) -> bool {
    matches!(
        value,
        "normal"
            | "stretch"
            | "flex-start"
            | "start"
            | "center"
            | "flex-end"
            | "end"
            | "space-between"
            | "space-around"
            | "space-evenly"
    )
}

fn is_supported_justify_content(value: &str) -> bool {
    is_supported_align_content(value)
}

#[cfg(test)]
mod tests {
    use super::apply_place_content;
    use crate::style::{FlexAlignment, TextStyle};

    #[test]
    fn place_content_sets_alignment_and_justification_and_rejects_invalid_values() {
        let mut style = TextStyle::default();
        apply_place_content(&mut style, "center space-between");
        assert_eq!(style.align_content, Some(FlexAlignment::Center));
        assert_eq!(style.justify_content, FlexAlignment::SpaceBetween);

        apply_place_content(&mut style, "space-evenly");
        assert_eq!(style.align_content, Some(FlexAlignment::SpaceEvenly));
        assert_eq!(style.justify_content, FlexAlignment::SpaceEvenly);

        apply_place_content(&mut style, "stretch");
        assert_eq!(style.align_content, Some(FlexAlignment::Stretch));
        assert_eq!(style.justify_content, FlexAlignment::FlexStart);

        apply_place_content(&mut style, "baseline center");
        assert_eq!(style.align_content, Some(FlexAlignment::Stretch));
        assert_eq!(style.justify_content, FlexAlignment::FlexStart);
    }
}
