use cosmic_text::{
    Align, Attrs, CacheKeyFlags, Color, Family, FeatureTag, FontFeatures, Metrics, Stretch, Style,
    Weight,
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
    pub(super) font_scale: f32,
    pub(super) line_height: f32,
    pub(super) line_height_multiplier: Option<f32>,
    pub(super) line_height_normal: bool,
    pub(super) color: [u8; 4],
    pub(super) color_opacity: f32,
    pub(super) background: [u8; 3],
    pub(super) family: String,
    pub(super) weight: Weight,
    pub(super) stretch: Stretch,
    pub(super) kerning: Option<bool>,
    pub(super) common_ligatures: Option<bool>,
    pub(super) discretionary_ligatures: Option<bool>,
    pub(super) historical_ligatures: Option<bool>,
    pub(super) contextual_alternates: Option<bool>,
    pub(super) font_style: Style,
    pub(super) font_variant_caps: FontVariantCaps,
    pub(super) font_variant_position: FontVariantPosition,
    pub(super) cursor: CursorIcon,
    pub(super) cursor_hidden: bool,
    pub(super) letter_spacing: f32,
    pub(super) letter_spacing_px: bool,
    pub(super) text_transform: TextTransform,
    pub(super) text_align: Align,
    pub(super) underline: bool,
    pub(super) decoration_style: TextDecorationStyle,
    pub(super) decoration_skip_ink: TextDecorationSkipInk,
    pub(super) decoration_thickness: Option<f32>,
    pub(super) underline_offset: Option<f32>,
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
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum FontVariantPosition {
    #[default]
    Normal,
    Sub,
    Super,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum TextDecorationStyle {
    #[default]
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum TextDecorationSkipInk {
    #[default]
    Auto,
    None,
    All,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: DEFAULT_FONT_SIZE,
            font_scale: 1.0,
            line_height: DEFAULT_LINE_HEIGHT,
            line_height_multiplier: Some(DEFAULT_LINE_HEIGHT / DEFAULT_FONT_SIZE),
            line_height_normal: true,
            color: [0, 0, 0, 255],
            color_opacity: 1.0,
            background: [255, 255, 255],
            family: "sans-serif".to_owned(),
            weight: Weight::NORMAL,
            stretch: Stretch::Normal,
            kerning: None,
            common_ligatures: None,
            discretionary_ligatures: None,
            historical_ligatures: None,
            contextual_alternates: None,
            font_style: Style::Normal,
            font_variant_caps: FontVariantCaps::Normal,
            font_variant_position: FontVariantPosition::Normal,
            cursor: CursorIcon::Text,
            cursor_hidden: false,
            letter_spacing: 0.0,
            letter_spacing_px: false,
            text_transform: TextTransform::None,
            text_align: Align::Left,
            underline: false,
            decoration_style: TextDecorationStyle::Solid,
            decoration_skip_ink: TextDecorationSkipInk::Auto,
            decoration_thickness: None,
            underline_offset: None,
            overline: false,
            strikethrough: false,
            text_decoration_color: None,
        }
    }
}

impl TextStyle {
    pub(super) fn font_features(&self) -> FontFeatures {
        let mut font_features = FontFeatures::new();
        if let Some(kerning) = self.kerning {
            font_features.set(FeatureTag::KERNING, u32::from(kerning));
        }
        if let Some(enabled) = self.common_ligatures {
            font_features.set(FeatureTag::STANDARD_LIGATURES, u32::from(enabled));
            font_features.set(FeatureTag::CONTEXTUAL_LIGATURES, u32::from(enabled));
        }
        if let Some(enabled) = self.discretionary_ligatures {
            font_features.set(FeatureTag::DISCRETIONARY_LIGATURES, u32::from(enabled));
        }
        if let Some(enabled) = self.historical_ligatures {
            font_features.set(FeatureTag::new(b"hlig"), u32::from(enabled));
        }
        if let Some(enabled) = self.contextual_alternates {
            font_features.set(FeatureTag::CONTEXTUAL_ALTERNATES, u32::from(enabled));
        }
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
            FontVariantCaps::PetiteCaps => {
                font_features.enable(FeatureTag::new(b"pcap"));
            }
            FontVariantCaps::AllPetiteCaps => {
                font_features
                    .enable(FeatureTag::new(b"pcap"))
                    .enable(FeatureTag::new(b"c2pc"));
            }
            FontVariantCaps::Unicase => {
                font_features.enable(FeatureTag::new(b"unic"));
            }
            FontVariantCaps::TitlingCaps => {
                font_features.enable(FeatureTag::new(b"titl"));
            }
        }
        match self.font_variant_position {
            FontVariantPosition::Normal => {}
            FontVariantPosition::Sub => {
                font_features.enable(FeatureTag::new(b"subs"));
            }
            FontVariantPosition::Super => {
                font_features.enable(FeatureTag::new(b"sups"));
            }
        }
        font_features
    }

    pub(super) fn to_attrs<'a>(&self, family_name: &'a str) -> Attrs<'a> {
        let family = match family_name.to_ascii_lowercase().as_str() {
            "serif" => Family::Serif,
            "monospace" => Family::Monospace,
            "cursive" => Family::Cursive,
            "fantasy" => Family::Fantasy,
            "sans-serif" => Family::SansSerif,
            _ => Family::Name(family_name),
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
        attrs = attrs.font_features(self.font_features());
        let mut decoration_metadata =
            (self.decoration_style as u64) | ((self.decoration_skip_ink as u64) << 4);
        if let Some(thickness) = self.decoration_thickness {
            let encoded_thickness = (thickness * 256.0).round().clamp(0.0, 0x00ff_ffff as f32);
            decoration_metadata |= 1 << 6;
            decoration_metadata |= (encoded_thickness as u64) << 8;
        }
        if let Some(offset) = self.underline_offset
            && usize::BITS >= 64
        {
            let encoded_offset = (offset * 16.0)
                .round()
                .clamp(i16::MIN as f32, i16::MAX as f32) as i16
                as u16;
            decoration_metadata |= 1_u64 << 32;
            decoration_metadata |= (encoded_offset as u64) << 33;
        }
        attrs = attrs.metadata(decoration_metadata as usize);
        attrs = attrs.metrics(Metrics::new(
            self.font_size * self.font_scale,
            self.line_height,
        ));
        if self.letter_spacing != 0.0 {
            attrs = attrs.letter_spacing(if self.letter_spacing_px {
                self.letter_spacing / self.font_size
            } else {
                self.letter_spacing
            });
        }
        if self.underline {
            attrs = attrs.underline(match self.decoration_style {
                TextDecorationStyle::Double => cosmic_text::UnderlineStyle::Double,
                _ => cosmic_text::UnderlineStyle::Single,
            });
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
                        push_variant_character(transformed, style, output);
                    }
                }
                TextTransform::Lowercase => {
                    for transformed in character.to_lowercase() {
                        push_variant_character(transformed, style, output);
                    }
                }
                TextTransform::Capitalize if *capitalize_next => {
                    for transformed in character.to_uppercase() {
                        push_variant_character(transformed, style, output);
                    }
                }
                _ => push_variant_character(character, style, output),
            }
            *last_was_space = false;
            *capitalize_next = !character.is_alphanumeric();
        }
    }
}

fn push_variant_character(character: char, style: &TextStyle, output: &mut Vec<TextRun>) {
    push_character(character, style, output);
}

pub(super) fn synthesize_font_variant_caps(run: TextRun) -> Vec<TextRun> {
    let mut output = Vec::new();
    for character in run.text.chars() {
        let synthesize = match run.style.font_variant_caps {
            FontVariantCaps::SmallCaps | FontVariantCaps::PetiteCaps => character.is_lowercase(),
            FontVariantCaps::AllSmallCaps | FontVariantCaps::AllPetiteCaps => {
                character.is_alphabetic()
            }
            _ => false,
        };
        let mut style = run.style.clone();
        style.font_variant_caps = FontVariantCaps::Normal;
        if synthesize {
            let original_size = style.font_size * style.font_scale;
            let synthesized_size = (original_size * 0.7).round();
            style.font_scale = if style.font_size > 0.0 {
                synthesized_size / style.font_size
            } else {
                0.0
            };
            for uppercase in character.to_uppercase() {
                push_character(uppercase, &style, &mut output);
            }
        } else {
            push_character(character, &style, &mut output);
        }
    }
    output
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

#[cfg(test)]
#[path = "style_tests.rs"]
mod tests;
