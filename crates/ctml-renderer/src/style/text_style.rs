use cosmic_text::{
    Attrs, CacheKeyFlags, Color, Family, FeatureTag, FontFeatures, Metrics, Style, Wrap,
};
use ctml_css::{FontRelativeUnits, ViewportSize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum UserSelect {
    #[default]
    Auto,
    Text,
    None,
    All,
}

mod core;
pub(crate) use core::TextStyle;

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
pub(crate) enum TextWrapMode {
    #[default]
    Wrap,
    NoWrap,
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
pub(crate) enum BorderLineStyle {
    #[default]
    None,
    Hidden,
    Solid,
    Dotted,
    Dashed,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Hyphens {
    None,
    #[default]
    Manual,
    Auto,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Overflow {
    #[default]
    Visible,
    Hidden,
    Clip,
    Auto,
    Scroll,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextRendering {
    #[default]
    Auto,
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TextOrientation {
    #[default]
    Mixed,
    Upright,
    Sideways,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalRl,
    VerticalLr,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum UnicodeBidi {
    #[default]
    Normal,
    Isolate,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayoutSize {
    pub(crate) node_id: usize,
    pub(crate) width: Option<String>,
    pub(crate) min_width: Option<String>,
    pub(crate) max_width: Option<String>,
    pub(crate) inline_size: Option<String>,
    pub(crate) min_inline_size: Option<String>,
    pub(crate) max_inline_size: Option<String>,
    pub(crate) height: Option<String>,
    pub(crate) min_height: Option<String>,
    pub(crate) max_height: Option<String>,
    pub(crate) block_size: Option<String>,
    pub(crate) min_block_size: Option<String>,
    pub(crate) max_block_size: Option<String>,
    pub(crate) border_box: bool,
    pub(crate) border_width: [Option<String>; 4],
    pub(crate) border_style: [BorderLineStyle; 4],
    pub(crate) border_color: [Option<[u8; 4]>; 4],
    pub(crate) border_radius: [[Option<String>; 2]; 4],
    pub(crate) padding: [Option<String>; 4],
    pub(crate) margin: [Option<String>; 4],
    pub(crate) intrinsic_min_content_width: Option<f32>,
    pub(crate) intrinsic_max_content_width: Option<f32>,
    pub(crate) overflow_x: Overflow,
    pub(crate) overflow_y: Overflow,
    pub(crate) size_property_orders: HashMap<String, usize>,
    pub(crate) writing_mode: WritingMode,
    pub(crate) font_size: f32,
    pub(crate) line_height: f32,
    pub(crate) font_relative_units: FontRelativeUnits,
    pub(crate) viewport_size: ViewportSize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LayoutHeight {
    pub(crate) width: Option<String>,
    pub(crate) min_width: Option<String>,
    pub(crate) max_width: Option<String>,
    pub(crate) inline_size: Option<String>,
    pub(crate) min_inline_size: Option<String>,
    pub(crate) max_inline_size: Option<String>,
    pub(crate) height: Option<String>,
    pub(crate) min_height: Option<String>,
    pub(crate) max_height: Option<String>,
    pub(crate) block_size: Option<String>,
    pub(crate) min_block_size: Option<String>,
    pub(crate) max_block_size: Option<String>,
    pub(crate) border_box: bool,
    pub(crate) border_width: [Option<String>; 4],
    pub(crate) border_style: [BorderLineStyle; 4],
    pub(crate) border_color: [Option<[u8; 4]>; 4],
    pub(crate) padding: [Option<String>; 4],
    pub(crate) margin: [Option<String>; 4],
    pub(crate) size_property_orders: HashMap<String, usize>,
    pub(crate) writing_mode: WritingMode,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextEmphasisSide {
    Left,
    Right,
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

impl TextStyle {
    pub(crate) fn wrap_mode(&self) -> Wrap {
        if self.text_wrap_mode == TextWrapMode::NoWrap {
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

    pub(crate) fn ellipsize_mode(&self) -> cosmic_text::Ellipsize {
        if let Some(lines) = self.line_clamp {
            cosmic_text::Ellipsize::End(cosmic_text::EllipsizeHeightLimit::Lines(lines))
        } else if self.text_overflow == TextOverflow::Ellipsis
            && self.text_wrap_mode == TextWrapMode::NoWrap
        {
            cosmic_text::Ellipsize::End(cosmic_text::EllipsizeHeightLimit::Lines(1))
        } else {
            cosmic_text::Ellipsize::None
        }
    }
}

impl TextStyle {
    pub(crate) fn font_features(&self) -> FontFeatures {
        let mut font_features = FontFeatures::new();
        let rendering_kerning = match self.text_rendering {
            TextRendering::Auto => None,
            TextRendering::OptimizeSpeed => Some(false),
            TextRendering::OptimizeLegibility | TextRendering::GeometricPrecision => Some(true),
        };
        if let Some(kerning) = self.kerning.or(rendering_kerning) {
            font_features.set(FeatureTag::KERNING, u32::from(kerning));
        }
        let rendering_ligatures = match self.text_rendering {
            TextRendering::Auto => None,
            TextRendering::OptimizeSpeed => Some(false),
            TextRendering::OptimizeLegibility | TextRendering::GeometricPrecision => Some(true),
        };
        if let Some(enabled) = self.common_ligatures.or(rendering_ligatures) {
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
        attrs = attrs.bidi_control(match self.bidi_isolate_direction {
            Some(Direction::Ltr) => Some(cosmic_text::BidiControl::IsolateLtr),
            Some(Direction::Rtl) => Some(cosmic_text::BidiControl::IsolateRtl),
            None => None,
        });
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
            variations.push(cosmic_text::FontVariation::new(
                *b"slnt",
                -self.font_oblique_angle.unwrap_or(14.0),
            ));
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
