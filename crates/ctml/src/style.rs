use cosmic_text::{
    Align, Attrs, CacheKeyFlags, Color, Family, FeatureTag, FontFeatures, Metrics, Stretch, Style,
    UnderlineStyle, Weight,
};
use winit::window::CursorIcon;

use crate::{
    StyleRule, UiNode,
    css_style::{self, ElementIdentity},
};

const DEFAULT_FONT_SIZE: f32 = 16.0;
const DEFAULT_LINE_HEIGHT: f32 = 20.0;

pub(super) fn compile_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
) -> (Vec<TextRun>, TextStyle) {
    let mut root_style = TextStyle::default();
    let body = ElementIdentity {
        tag: "body".to_owned(),
        class: None,
        id: None,
    };
    css_style::apply_matching_styles(&mut root_style, &body, &[], stylesheet);
    let mut runs = Vec::new();
    let mut path = vec![body];
    let mut last_was_space = true;
    let mut capitalize_next = true;
    collect_runs(
        nodes,
        stylesheet,
        &mut path,
        &root_style,
        &mut runs,
        &mut last_was_space,
        &mut capitalize_next,
    );
    trim_runs(&mut runs);
    (runs, root_style)
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TextStyle {
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) color: [u8; 4],
    pub(super) background: [u8; 3],
    pub(super) family: String,
    pub(super) weight: Weight,
    pub(super) stretch: Stretch,
    pub(super) font_style: Style,
    pub(super) font_variant_caps: FontVariantCaps,
    pub(super) cursor: CursorIcon,
    pub(super) letter_spacing: f32,
    pub(super) letter_spacing_px: bool,
    pub(super) text_transform: TextTransform,
    pub(super) text_align: Align,
    pub(super) underline: bool,
    pub(super) overline: bool,
    pub(super) strikethrough: bool,
    pub(super) text_decoration_color: Option<[u8; 4]>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum FontVariantCaps {
    #[default]
    Normal,
    SmallCaps,
    AllSmallCaps,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: DEFAULT_FONT_SIZE,
            line_height: DEFAULT_LINE_HEIGHT,
            color: [0, 0, 0, 255],
            background: [255, 255, 255],
            family: "sans-serif".to_owned(),
            weight: Weight::NORMAL,
            stretch: Stretch::Normal,
            font_style: Style::Normal,
            font_variant_caps: FontVariantCaps::Normal,
            cursor: CursorIcon::Text,
            letter_spacing: 0.0,
            letter_spacing_px: false,
            text_transform: TextTransform::None,
            text_align: Align::Left,
            underline: false,
            overline: false,
            strikethrough: false,
            text_decoration_color: None,
        }
    }
}

impl TextStyle {
    pub(super) fn to_attrs(&self) -> Attrs<'_> {
        let family = match self.family.to_ascii_lowercase().as_str() {
            "serif" => Family::Serif,
            "monospace" => Family::Monospace,
            "cursive" => Family::Cursive,
            "fantasy" => Family::Fantasy,
            "sans-serif" => Family::SansSerif,
            _ => Family::Name(&self.family),
        };
        let mut attrs = Attrs::new()
            .family(family)
            .cache_key_flags(CacheKeyFlags::DISABLE_HINTING)
            .color(Color::rgba(
                self.color[0],
                self.color[1],
                self.color[2],
                self.color[3],
            ))
            .weight(self.weight)
            .stretch(self.stretch)
            .style(self.font_style);
        let mut font_features = FontFeatures::new();
        match self.font_variant_caps {
            FontVariantCaps::Normal => {}
            FontVariantCaps::SmallCaps => {
                font_features.enable(FeatureTag::SMALL_CAPS);
            }
            FontVariantCaps::AllSmallCaps => {
                font_features
                    .enable(FeatureTag::SMALL_CAPS)
                    .enable(FeatureTag::ALL_SMALL_CAPS);
            }
        }
        attrs = attrs.font_features(font_features);
        attrs = attrs.metrics(Metrics::new(self.font_size, self.line_height));
        if self.letter_spacing != 0.0 {
            attrs = attrs.letter_spacing(if self.letter_spacing_px {
                self.letter_spacing / self.font_size
            } else {
                self.letter_spacing
            });
        }
        if self.underline {
            attrs = attrs.underline(UnderlineStyle::Single);
        }
        if let Some(color) = self.text_decoration_color {
            let color = Color::rgba(color[0], color[1], color[2], color[3]);
            if self.underline {
                attrs = attrs.underline_color(color);
            }
            if self.strikethrough {
                attrs = attrs.strikethrough_color(color);
            }
            if self.overline {
                attrs = attrs.overline_color(color);
            }
        }
        if self.strikethrough {
            attrs = attrs.strikethrough();
        }
        if self.overline {
            attrs = attrs.overline();
        }
        attrs
    }
}

pub(super) struct TextRun {
    pub(super) text: String,
    pub(super) style: TextStyle,
}

fn collect_runs(
    nodes: &[UiNode],
    stylesheet: &[StyleRule],
    path: &mut Vec<ElementIdentity>,
    inherited: &TextStyle,
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    for node in nodes {
        match node {
            UiNode::Text(text) => {
                append_text(text, inherited, output, last_was_space, capitalize_next)
            }
            UiNode::Element {
                tag,
                class,
                id,
                children,
            } => {
                let identity = ElementIdentity {
                    tag: tag.clone(),
                    class: class.clone(),
                    id: id.clone(),
                };
                let style = css_style::resolve_style(inherited, &identity, path, stylesheet);
                let block = is_block(tag);
                if block {
                    push_line_break(output, &style, last_was_space);
                    *capitalize_next = true;
                }
                path.push(identity);
                collect_runs(
                    children,
                    stylesheet,
                    path,
                    &style,
                    output,
                    last_was_space,
                    capitalize_next,
                );
                path.pop();
                if block {
                    push_line_break(output, &style, last_was_space);
                    *capitalize_next = true;
                }
            }
        }
    }
}

fn append_text(
    text: &str,
    style: &TextStyle,
    output: &mut Vec<TextRun>,
    last_was_space: &mut bool,
    capitalize_next: &mut bool,
) {
    for character in text.chars() {
        if character.is_whitespace() {
            if !*last_was_space {
                push_character(' ', style, output);
            }
            *last_was_space = true;
            *capitalize_next = true;
        } else {
            match style.text_transform {
                TextTransform::Uppercase => {
                    for transformed in character.to_uppercase() {
                        push_character(transformed, style, output);
                    }
                }
                TextTransform::Lowercase => {
                    for transformed in character.to_lowercase() {
                        push_character(transformed, style, output);
                    }
                }
                TextTransform::Capitalize if *capitalize_next => {
                    for transformed in character.to_uppercase() {
                        push_character(transformed, style, output);
                    }
                }
                _ => push_character(character, style, output),
            }
            *last_was_space = false;
            *capitalize_next = !character.is_alphanumeric();
        }
    }
}

fn push_line_break(output: &mut Vec<TextRun>, style: &TextStyle, last_was_space: &mut bool) {
    if let Some(last) = output.last_mut() {
        while last.text.ends_with(' ') {
            last.text.pop();
        }
        if !last.text.ends_with('\n') {
            push_character('\n', style, output);
        }
    }
    *last_was_space = true;
}

fn push_character(character: char, style: &TextStyle, output: &mut Vec<TextRun>) {
    if let Some(last) = output.last_mut()
        && last.style == *style
    {
        last.text.push(character);
    } else {
        output.push(TextRun {
            text: character.to_string(),
            style: style.clone(),
        });
    }
}

fn trim_runs(runs: &mut Vec<TextRun>) {
    while runs
        .first()
        .is_some_and(|run| run.text.chars().all(char::is_whitespace))
    {
        runs.remove(0);
    }
    while runs
        .last()
        .is_some_and(|run| run.text.chars().all(char::is_whitespace))
    {
        runs.pop();
    }
    if let Some(last) = runs.last_mut() {
        let trimmed = last.text.trim_end().to_owned();
        last.text = trimmed;
    }
    runs.retain(|run| !run.text.is_empty());
}

fn is_block(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "div"
            | "footer"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "section"
            | "ul"
    )
}
