use cosmic_text::{
    Align, Attrs, CacheKeyFlags, Color, Family, FeatureTag, FontFeatures, Metrics, Stretch, Style,
    Weight,
};
use std::collections::HashMap;
use winit::window::CursorIcon;

const DEFAULT_FONT_SIZE: f32 = 16.0;
const DEFAULT_LINE_HEIGHT: f32 = 20.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextStyle {
    pub(crate) font_size: f32,
    pub(crate) font_scale: f32,
    pub(crate) line_height: f32,
    pub(crate) line_height_multiplier: Option<f32>,
    pub(crate) line_height_normal: bool,
    pub(crate) color: [u8; 4],
    pub(crate) color_opacity: f32,
    pub(crate) background: [u8; 3],
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
    pub(crate) cursor: CursorIcon,
    pub(crate) cursor_hidden: bool,
    pub(crate) letter_spacing: f32,
    pub(crate) letter_spacing_px: bool,
    pub(crate) text_transform: TextTransform,
    pub(crate) text_align: Align,
    pub(crate) underline: bool,
    pub(crate) decoration_style: TextDecorationStyle,
    pub(crate) decoration_skip_ink: TextDecorationSkipInk,
    pub(crate) decoration_thickness: Option<f32>,
    pub(crate) underline_offset: Option<f32>,
    pub(crate) overline: bool,
    pub(crate) strikethrough: bool,
    pub(crate) text_decoration_color: Option<[u8; 4]>,
    pub(crate) custom_properties: HashMap<String, String>,
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
            font_variant_numeric: Vec::new(),
            font_variant_east_asian: Vec::new(),
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
            custom_properties: HashMap::new(),
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
