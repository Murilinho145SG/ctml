use cosmic_text::{
    Align, Attrs, CacheKeyFlags, Color, Family, FeatureTag, FontFeatures, Metrics, Stretch, Style,
    Weight, Wrap,
};
use ctml_css::{FontRelativeUnits, ViewportSize};
use std::collections::HashMap;
use winit::window::CursorIcon;

const DEFAULT_FONT_SIZE: f32 = 16.0;
const DEFAULT_LINE_HEIGHT: f32 = 20.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum WhiteSpace {
    #[default]
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum OverflowWrap {
    #[default]
    Normal,
    BreakWord,
    Anywhere,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum WordBreak {
    #[default]
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum LineBreak {
    #[default]
    Auto,
    Loose,
    Normal,
    Strict,
    Anywhere,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextStyle {
    pub(crate) font_relative_units: FontRelativeUnits,
    pub(crate) viewport_size: ViewportSize,
    pub(crate) font_size: f32,
    pub(crate) font_scale: f32,
    pub(crate) line_height: f32,
    pub(crate) line_height_multiplier: Option<f32>,
    pub(crate) line_height_normal: bool,
    pub(crate) vertical_align_offset: f32,
    pub(crate) color: [u8; 4],
    pub(crate) color_opacity: f32,
    pub(crate) opacity: f32,
    pub(crate) background: [u8; 3],
    pub(crate) background_color: Option<[u8; 4]>,
    pub(crate) background_layers: Vec<[u8; 4]>,
    pub(crate) display_none: bool,
    pub(crate) display_block: Option<bool>,
    pub(crate) display_list_item: Option<bool>,
    pub(crate) list_style_type: Option<String>,
    pub(crate) width: Option<String>,
    pub(crate) min_width: Option<String>,
    pub(crate) max_width: Option<String>,
    pub(crate) border_box: bool,
    pub(crate) layout_sizes: Vec<LayoutSize>,
    pub(crate) height: Option<String>,
    pub(crate) min_height: Option<String>,
    pub(crate) max_height: Option<String>,
    pub(crate) empty_block_height: Option<LayoutHeight>,
    pub(crate) layout_end_sizes: Vec<LayoutSize>,
    pub(crate) flow_spacer: bool,
    pub(crate) family: String,
    pub(crate) weight: Weight,
    pub(crate) stretch: Stretch,
    pub(crate) kerning: Option<bool>,
    pub(crate) common_ligatures: Option<bool>,
    pub(crate) discretionary_ligatures: Option<bool>,
    pub(crate) historical_ligatures: Option<bool>,
    pub(crate) contextual_alternates: Option<bool>,
    pub(crate) font_style: Style,
    pub(crate) font_variant_caps: FontVariantCaps,
    pub(crate) font_variant_position: FontVariantPosition,
    pub(crate) font_variant_numeric: Vec<String>,
    pub(crate) font_variant_east_asian: Vec<String>,
    pub(crate) font_feature_settings: Vec<([u8; 4], u32)>,
    pub(crate) font_optical_sizing: bool,
    pub(crate) font_variation_settings: Vec<([u8; 4], f32)>,
    pub(crate) cursor: CursorIcon,
    pub(crate) cursor_hidden: bool,
    pub(crate) letter_spacing: f32,
    pub(crate) letter_spacing_px: bool,
    pub(crate) word_spacing_px: f32,
    pub(crate) word_spacing_percent: Option<f32>,
    pub(crate) word_spacing_space: bool,
    pub(crate) text_transform: TextTransform,
    pub(crate) text_align: Align,
    pub(crate) text_align_last: Option<Align>,
    pub(crate) text_justify: cosmic_text::TextJustify,
    pub(crate) white_space: WhiteSpace,
    pub(crate) overflow_wrap: OverflowWrap,
    pub(crate) word_break: WordBreak,
    pub(crate) line_break: LineBreak,
    pub(crate) text_indent: f32,
    pub(crate) text_indent_percent: f32,
    pub(crate) text_indent_hanging: bool,
    pub(crate) text_indent_each_line: bool,
    pub(crate) underline: bool,
    pub(crate) decoration_style: TextDecorationStyle,
    pub(crate) decoration_skip_ink: TextDecorationSkipInk,
    pub(crate) decoration_thickness: Option<f32>,
    pub(crate) underline_offset: Option<f32>,
    pub(crate) overline: bool,
    pub(crate) strikethrough: bool,
    pub(crate) text_decoration_color: Option<[u8; 4]>,
    pub(crate) text_shadows: Vec<TextShadow>,
    pub(crate) text_emphasis_style: TextEmphasisStyle,
    pub(crate) text_emphasis_color: Option<[u8; 4]>,
    pub(crate) text_emphasis_position: TextEmphasisPosition,
    pub(crate) custom_properties: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayoutSize {
    pub(crate) node_id: usize,
    pub(crate) width: Option<String>,
    pub(crate) min_width: Option<String>,
    pub(crate) max_width: Option<String>,
    pub(crate) height: Option<String>,
    pub(crate) min_height: Option<String>,
    pub(crate) max_height: Option<String>,
    pub(crate) font_size: f32,
    pub(crate) line_height: f32,
    pub(crate) font_relative_units: FontRelativeUnits,
    pub(crate) viewport_size: ViewportSize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayoutHeight {
    pub(crate) height: Option<String>,
    pub(crate) min_height: Option<String>,
    pub(crate) max_height: Option<String>,
    pub(crate) font_size: f32,
    pub(crate) line_height: f32,
    pub(crate) font_relative_units: FontRelativeUnits,
    pub(crate) viewport_size: ViewportSize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextShadow {
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
    pub(crate) blur: f32,
    pub(crate) color: [u8; 4],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextEmphasisStyle {
    #[default]
    None,
    FilledDot,
    OpenDot,
    FilledCircle,
    OpenCircle,
    FilledDoubleCircle,
    OpenDoubleCircle,
    FilledTriangle,
    OpenTriangle,
    FilledSesame,
    OpenSesame,
    Custom(String),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextEmphasisPosition {
    #[default]
    Over,
    Under,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FontVariantCaps {
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
pub(crate) enum FontVariantPosition {
    #[default]
    Normal,
    Sub,
    Super,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum TextDecorationStyle {
    #[default]
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum TextDecorationSkipInk {
    #[default]
    Auto,
    None,
    All,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_relative_units: FontRelativeUnits::default(),
            viewport_size: ViewportSize::default(),
            font_size: DEFAULT_FONT_SIZE,
            font_scale: 1.0,
            line_height: DEFAULT_LINE_HEIGHT,
            line_height_multiplier: Some(DEFAULT_LINE_HEIGHT / DEFAULT_FONT_SIZE),
            line_height_normal: true,
            vertical_align_offset: 0.0,
            color: [0, 0, 0, 255],
            color_opacity: 1.0,
            opacity: 1.0,
            background: [255, 255, 255],
            background_color: None,
            background_layers: Vec::new(),
            display_none: false,
            display_block: None,
            display_list_item: None,
            list_style_type: None,
            width: None,
            min_width: None,
            max_width: None,
            border_box: false,
            layout_sizes: Vec::new(),
            height: None,
            min_height: None,
            max_height: None,
            empty_block_height: None,
            layout_end_sizes: Vec::new(),
            flow_spacer: false,
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
            font_variant_numeric: Vec::new(),
            font_variant_east_asian: Vec::new(),
            font_feature_settings: Vec::new(),
            font_optical_sizing: true,
            font_variation_settings: Vec::new(),
            cursor: CursorIcon::Text,
            cursor_hidden: false,
            letter_spacing: 0.0,
            letter_spacing_px: false,
            word_spacing_px: 0.0,
            word_spacing_percent: None,
            word_spacing_space: false,
            text_transform: TextTransform::None,
            text_align: Align::Left,
            text_align_last: None,
            text_justify: cosmic_text::TextJustify::Auto,
            white_space: WhiteSpace::Normal,
            overflow_wrap: OverflowWrap::Normal,
            word_break: WordBreak::Normal,
            line_break: LineBreak::Auto,
            text_indent: 0.0,
            text_indent_percent: 0.0,
            text_indent_hanging: false,
            text_indent_each_line: false,
            underline: false,
            decoration_style: TextDecorationStyle::Solid,
            decoration_skip_ink: TextDecorationSkipInk::Auto,
            decoration_thickness: None,
            underline_offset: None,
            overline: false,
            strikethrough: false,
            text_decoration_color: None,
            text_shadows: Vec::new(),
            text_emphasis_style: TextEmphasisStyle::None,
            text_emphasis_color: None,
            text_emphasis_position: TextEmphasisPosition::Over,
            custom_properties: HashMap::new(),
        }
    }
}

impl TextStyle {
    pub(crate) fn wrap_mode(&self) -> Wrap {
        if matches!(self.white_space, WhiteSpace::NoWrap | WhiteSpace::Pre) {
            return Wrap::None;
        }
        if self.line_break == LineBreak::Anywhere {
            return Wrap::Glyph;
        }
        match self.word_break {
            WordBreak::BreakAll => Wrap::Glyph,
            WordBreak::BreakWord => Wrap::WordOrGlyph,
            WordBreak::Normal | WordBreak::KeepAll => match self.overflow_wrap {
                OverflowWrap::Normal => Wrap::Word,
                OverflowWrap::BreakWord | OverflowWrap::Anywhere => Wrap::WordOrGlyph,
            },
        }
    }
}

impl TextStyle {
    pub(crate) fn font_features(&self) -> FontFeatures {
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
        for value in &self.font_variant_numeric {
            let tag = match value.as_str() {
                "lining-nums" => Some(*b"lnum"),
                "oldstyle-nums" => Some(*b"onum"),
                "proportional-nums" => Some(*b"pnum"),
                "tabular-nums" => Some(*b"tnum"),
                "diagonal-fractions" => Some(*b"frac"),
                "stacked-fractions" => Some(*b"afrc"),
                "ordinal" => Some(*b"ordn"),
                "slashed-zero" => Some(*b"zero"),
                _ => None,
            };
            if let Some(tag) = tag {
                font_features.enable(FeatureTag::new(&tag));
            }
        }
        for value in &self.font_variant_east_asian {
            let tag = match value.as_str() {
                "jis78" => Some(*b"jp78"),
                "jis83" => Some(*b"jp83"),
                "jis90" => Some(*b"jp90"),
                "jis04" => Some(*b"jp04"),
                "simplified" => Some(*b"smpl"),
                "traditional" => Some(*b"trad"),
                "full-width" => Some(*b"fwid"),
                "proportional-width" => Some(*b"pwid"),
                "ruby" => Some(*b"ruby"),
                _ => None,
            };
            if let Some(tag) = tag {
                font_features.enable(FeatureTag::new(&tag));
            }
        }
        for (tag, value) in &self.font_feature_settings {
            font_features.set(FeatureTag::new(tag), *value);
        }
        font_features
    }

    pub(crate) fn to_attrs<'a>(&self, family_name: &'a str) -> Attrs<'a> {
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
        let mut variations = Vec::new();
        if self.font_optical_sizing
            && !self
                .font_variation_settings
                .iter()
                .any(|(tag, _)| *tag == *b"opsz")
        {
            variations.push(cosmic_text::FontVariation::new(*b"opsz", self.font_size));
        }
        if self.font_style == Style::Italic
            && !self
                .font_variation_settings
                .iter()
                .any(|(tag, _)| *tag == *b"ital")
        {
            variations.push(cosmic_text::FontVariation::new(*b"ital", 1.0));
        } else if self.font_style == Style::Oblique
            && !self
                .font_variation_settings
                .iter()
                .any(|(tag, _)| *tag == *b"slnt")
        {
            variations.push(cosmic_text::FontVariation::new(*b"slnt", -14.0));
        }
        variations.extend(
            self.font_variation_settings
                .iter()
                .map(|(tag, value)| cosmic_text::FontVariation::new(*tag, *value)),
        );
        attrs = attrs.font_variations(variations);
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
        let mut decoration_color = self.text_decoration_color.unwrap_or(self.color);
        decoration_color[3] = (f32::from(decoration_color[3]) * self.opacity)
            .round()
            .clamp(0.0, 255.0) as u8;
        let decoration_color = Color::rgba(
            decoration_color[0],
            decoration_color[1],
            decoration_color[2],
            decoration_color[3],
        );
        if self.underline {
            attrs = attrs.underline_color(decoration_color);
        }
        if self.strikethrough {
            attrs = attrs.strikethrough_color(decoration_color);
        }
        if self.overline {
            attrs = attrs.overline_color(decoration_color);
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
