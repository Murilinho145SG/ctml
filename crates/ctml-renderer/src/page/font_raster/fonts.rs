use crate::style::TextStyle;
use cosmic_text::{FontSystem, fontdb};
use skia_safe::{
    Font, FontHinting, FontMgr,
    font::Edging,
    font_style::{FontStyle as SkiaFontStyle, Slant, Weight as SkiaWeight, Width as SkiaWidth},
};

pub(super) fn configure_font(font: &mut Font) {
    font.set_edging(Edging::AntiAlias);
    font.set_hinting(FontHinting::None);
    font.set_force_auto_hinting(true);
    font.set_embedded_bitmaps(true);
    font.set_subpixel(true);
    font.set_linear_metrics(true);
}

pub(super) fn apply_synthetic_style(font: &mut Font, face: &fontdb::FaceInfo, style: &TextStyle) {
    font.set_embolden(
        style.weight >= fontdb::Weight::SEMIBOLD && face.weight < fontdb::Weight::SEMIBOLD,
    );
    font.set_skew_x(
        if style.font_style != fontdb::Style::Normal && face.style == fontdb::Style::Normal {
            -0.25
        } else {
            0.0
        },
    );
}

pub(super) fn load_typeface(
    font_system: &FontSystem,
    font_manager: &FontMgr,
    font_id: fontdb::ID,
) -> Option<skia_safe::Typeface> {
    let from_database = crate::typeface_cache::load(font_system, font_id);
    if from_database.is_some() {
        return from_database;
    }

    let matched = font_system.db().face(font_id).and_then(|face| {
        let family = face.families.first()?.0.as_str();
        let width = match face.stretch {
            fontdb::Stretch::UltraCondensed => SkiaWidth::ULTRA_CONDENSED,
            fontdb::Stretch::ExtraCondensed => SkiaWidth::EXTRA_CONDENSED,
            fontdb::Stretch::Condensed => SkiaWidth::CONDENSED,
            fontdb::Stretch::SemiCondensed => SkiaWidth::SEMI_CONDENSED,
            fontdb::Stretch::Normal => SkiaWidth::NORMAL,
            fontdb::Stretch::SemiExpanded => SkiaWidth::SEMI_EXPANDED,
            fontdb::Stretch::Expanded => SkiaWidth::EXPANDED,
            fontdb::Stretch::ExtraExpanded => SkiaWidth::EXTRA_EXPANDED,
            fontdb::Stretch::UltraExpanded => SkiaWidth::ULTRA_EXPANDED,
        };
        let slant = match face.style {
            fontdb::Style::Normal => Slant::Upright,
            fontdb::Style::Italic => Slant::Italic,
            fontdb::Style::Oblique => Slant::Oblique,
        };
        let style = SkiaFontStyle::new(SkiaWeight::from(i32::from(face.weight.0)), width, slant);
        font_manager.match_family_style(family, style)
    });
    matched
}
