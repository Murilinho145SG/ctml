use cosmic_text::{
    FontSystem, fontdb,
    fontdb::{Family as DatabaseFamily, Query as FontQuery},
};
use skia_safe::{Font, FontHinting, FontMgr};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::{font_family::resolve_family_list, style::TextStyle};

pub(crate) fn font_relative_units(
    font_system: &FontSystem,
    style: &TextStyle,
) -> ctml_css::FontRelativeUnits {
    let family_name = resolve_family_list(&style.family, font_system);
    let family = match family_name.to_ascii_lowercase().as_str() {
        "serif" => DatabaseFamily::Serif,
        "sans-serif" => DatabaseFamily::SansSerif,
        "monospace" => DatabaseFamily::Monospace,
        "cursive" => DatabaseFamily::Cursive,
        "fantasy" => DatabaseFamily::Fantasy,
        _ => DatabaseFamily::Name(&family_name),
    };
    let families = [family];
    let Some(id) = font_system.db().query(&FontQuery {
        families: &families,
        weight: style.weight,
        stretch: style.stretch,
        style: match style.font_style {
            cosmic_text::Style::Normal => fontdb::Style::Normal,
            cosmic_text::Style::Italic => fontdb::Style::Italic,
            cosmic_text::Style::Oblique => fontdb::Style::Oblique,
        },
    }) else {
        return ctml_css::FontRelativeUnits::default();
    };
    let font_manager = FontMgr::new();
    let Some(typeface) = font_system
        .db()
        .with_face_data(id, |data, index| {
            font_manager.new_from_bytes(data, Some(index))
        })
        .flatten()
    else {
        return ctml_css::FontRelativeUnits::default();
    };
    let font_size = style.font_size.max(1.0);
    let mut font = Font::from_typeface(typeface, font_size);
    font.set_hinting(FontHinting::None);
    font.set_force_auto_hinting(true);
    font.set_subpixel(true);
    font.set_linear_metrics(true);
    let (_, metrics) = font.metrics();
    let measure_advance = |character: char| {
        (font.unichar_to_glyph(character as i32) != 0).then_some(())?;
        let text = character.to_string();
        let (advance, _) = font.measure_text(text, None);
        (advance.is_finite() && advance > 0.0).then_some(advance / font_size)
    };
    let fallback = ctml_css::FontRelativeUnits::default();
    ctml_css::FontRelativeUnits {
        ex: positive_metric(metrics.x_height, font_size)
            .map_or(fallback.ex, |metric| metric / font_size),
        ch: measure_advance('0').unwrap_or(fallback.ch),
        ic: measure_advance('水').unwrap_or(fallback.ic),
        cap: positive_metric(metrics.cap_height, font_size)
            .or_else(|| positive_metric(-metrics.ascent, font_size))
            .map_or(fallback.cap, |metric| metric / font_size),
    }
}

pub(crate) struct FontRelativeUnitsContext<'a> {
    font_system: &'a FontSystem,
    cache: RefCell<HashMap<String, ctml_css::FontRelativeUnits>>,
}

impl<'a> FontRelativeUnitsContext<'a> {
    pub(crate) fn new(font_system: &'a FontSystem) -> Self {
        Self {
            font_system,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn resolve(&self, style: &TextStyle) -> ctml_css::FontRelativeUnits {
        let family = resolve_family_list(&style.family, self.font_system);
        let key = format!(
            "{family}|{}|{:?}|{:?}|{}",
            style.weight.0,
            style.stretch,
            style.font_style,
            style.font_size.to_bits()
        );
        if let Some(metrics) = self.cache.borrow().get(&key) {
            return *metrics;
        }
        let metrics = font_relative_units(self.font_system, style);
        self.cache.borrow_mut().insert(key, metrics);
        metrics
    }
}

fn positive_metric(value: f32, font_size: f32) -> Option<f32> {
    let value = value.abs();
    (value.is_finite() && value > 0.0).then_some(value.min(font_size))
}

pub(crate) fn normal_line_height(
    font_system: &mut FontSystem,
    style: &TextStyle,
    family: &str,
) -> Option<f32> {
    let family = match family.to_ascii_lowercase().as_str() {
        "serif" => DatabaseFamily::Serif,
        "sans-serif" => DatabaseFamily::SansSerif,
        "monospace" => DatabaseFamily::Monospace,
        "cursive" => DatabaseFamily::Cursive,
        "fantasy" => DatabaseFamily::Fantasy,
        _ => DatabaseFamily::Name(family),
    };
    let families = [family];
    let id = font_system.db().query(&FontQuery {
        families: &families,
        weight: style.weight,
        stretch: style.stretch,
        style: match style.font_style {
            cosmic_text::Style::Normal => cosmic_text::fontdb::Style::Normal,
            cosmic_text::Style::Italic => cosmic_text::fontdb::Style::Italic,
            cosmic_text::Style::Oblique => cosmic_text::fontdb::Style::Oblique,
        },
    })?;
    let font_manager = FontMgr::new();
    let typeface = font_system
        .db()
        .with_face_data(id, |data, index| {
            font_manager.new_from_bytes(data, Some(index))
        })
        .flatten()?;
    let mut font = Font::from_typeface(typeface, style.font_size);
    font.set_hinting(FontHinting::None);
    font.set_force_auto_hinting(true);
    font.set_subpixel(true);
    font.set_linear_metrics(true);
    let (_, metrics) = font.metrics();
    Some(normal_line_height_from_metrics(
        -metrics.ascent,
        metrics.descent,
        metrics.leading,
        false,
        true,
    ))
}

fn normal_line_height_from_metrics(
    ascent: f32,
    descent: f32,
    leading: f32,
    subpixel_ascent_descent: bool,
    subpixel_positioning: bool,
) -> f32 {
    let (ascent, descent) = if subpixel_ascent_descent && (ascent < 3.0 || ascent + descent < 2.0) {
        (ascent, descent)
    } else {
        let mut ascent = ascent.round();
        let descent_metric = descent;
        let mut descent = descent.round();
        if descent < descent_metric && subpixel_positioning && ascent >= 1.0 {
            ascent -= 1.0;
            descent += 1.0;
        }
        (ascent, descent)
    };
    ascent.round() + descent.round() + leading.round()
}

#[cfg(test)]
mod tests {
    use super::{font_relative_units, normal_line_height_from_metrics};
    use crate::style::TextStyle;
    use cosmic_text::FontSystem;

    #[test]
    fn measures_font_relative_units_from_the_selected_typeface() {
        let font_system = FontSystem::new();
        let units = font_relative_units(&font_system, &TextStyle::default());

        assert!(units.ex.is_finite() && units.ex > 0.0);
        assert!(units.ch.is_finite() && units.ch > 0.0);
        assert!(units.ic.is_finite() && units.ic > 0.0);
        assert!(units.cap.is_finite() && units.cap > 0.0);
    }

    #[test]
    fn rounds_metrics_without_a_fixed_line_height_floor() {
        let line_height = normal_line_height_from_metrics(16.3, 3.8, 0.6, false, true);

        assert_eq!(line_height, 21.0);
        assert!(line_height < 18.0 * 1.2);
    }

    #[test]
    fn rounds_each_font_metric_before_adding_line_height() {
        let line_height = normal_line_height_from_metrics(10.2, 2.2, 0.2, false, true);

        assert_eq!(line_height, 12.0);
    }
}
