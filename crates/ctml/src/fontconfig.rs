use std::ffi::{CStr, CString, c_char};

use fontconfig_sys as fc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FontRenderParams {
    pub(crate) antialiasing: bool,
    pub(crate) hinting: i32,
    pub(crate) autohinter: bool,
    pub(crate) use_bitmaps: bool,
}

pub(crate) fn matched_family(requested_family: &str) -> Option<String> {
    // Fontconfig owns the returned patterns and strings; this wrapper keeps their lifetime local.
    unsafe {
        let request = Pattern(fc::FcPatternCreate());
        if request.0.is_null() {
            return None;
        }

        let requested_family = CString::new(requested_family).ok()?;
        if fc::FcPatternAddString(
            request.0,
            fc::constants::FC_FAMILY.as_ptr(),
            requested_family.as_ptr().cast(),
        ) == 0
        {
            return None;
        }
        if fc::FcConfigSubstitute(std::ptr::null_mut(), request.0, fc::FcMatchPattern) == 0 {
            return None;
        }
        fc::FcDefaultSubstitute(request.0);

        let mut result = fc::FcResultNoMatch;
        let matched = Pattern(fc::FcFontMatch(
            std::ptr::null_mut(),
            request.0,
            &mut result,
        ));
        if matched.0.is_null() {
            return None;
        }

        let mut family = std::ptr::null_mut();
        if fc::FcPatternGetString(matched.0, fc::constants::FC_FAMILY.as_ptr(), 0, &mut family)
            != fc::FcResultMatch as u32
            || family.is_null()
        {
            return None;
        }

        Some(
            CStr::from_ptr(family.cast::<c_char>())
                .to_string_lossy()
                .into_owned(),
        )
    }
}

pub(crate) fn render_params(
    family: &str,
    pixel_size: f32,
    weight: u16,
    italic: bool,
) -> Option<FontRenderParams> {
    unsafe {
        let request = Pattern(fc::FcPatternCreate());
        if request.0.is_null() {
            return None;
        }

        let family = CString::new(family).ok()?;
        let added = fc::FcPatternAddString(
            request.0,
            fc::constants::FC_FAMILY.as_ptr(),
            family.as_ptr().cast(),
        ) != 0
            && fc::FcPatternAddString(request.0, c"fontwrapper".as_ptr(), c"SFNT".as_ptr().cast())
                != 0
            && fc::FcPatternAddBool(request.0, fc::constants::FC_SCALABLE.as_ptr(), 1) != 0
            && fc::FcPatternAddDouble(
                request.0,
                fc::constants::FC_PIXEL_SIZE.as_ptr(),
                f64::from(pixel_size),
            ) != 0
            && fc::FcPatternAddInteger(
                request.0,
                fc::constants::FC_WEIGHT.as_ptr(),
                fontconfig_weight(weight),
            ) != 0
            && fc::FcPatternAddInteger(
                request.0,
                fc::constants::FC_SLANT.as_ptr(),
                if italic {
                    fc::constants::FC_SLANT_ITALIC
                } else {
                    fc::constants::FC_SLANT_ROMAN
                },
            ) != 0;
        if !added
            || fc::FcConfigSubstitute(std::ptr::null_mut(), request.0, fc::FcMatchPattern) == 0
        {
            return None;
        }
        fc::FcDefaultSubstitute(request.0);

        let mut result = fc::FcResultNoMatch;
        let matched = Pattern(fc::FcFontMatch(
            std::ptr::null_mut(),
            request.0,
            &mut result,
        ));
        if matched.0.is_null() {
            return None;
        }

        let mut params = FontRenderParams {
            antialiasing: true,
            hinting: fc::constants::FC_HINT_SLIGHT,
            autohinter: false,
            use_bitmaps: true,
        };
        get_bool(
            matched.0,
            fc::constants::FC_ANTIALIAS,
            &mut params.antialiasing,
        );
        get_bool(
            matched.0,
            fc::constants::FC_AUTOHINT,
            &mut params.autohinter,
        );
        get_bool(
            matched.0,
            fc::constants::FC_EMBEDDED_BITMAP,
            &mut params.use_bitmaps,
        );

        let mut hinting = 0;
        if get_bool_value(matched.0, fc::constants::FC_HINTING).unwrap_or(true) {
            get_integer(matched.0, fc::constants::FC_HINT_STYLE, &mut hinting);
        }
        params.hinting = hinting;

        if !params.antialiasing {
            params.hinting = fc::constants::FC_HINT_FULL;
        }

        Some(params)
    }
}

fn fontconfig_weight(weight: u16) -> i32 {
    use fc::constants as weight_fc;

    match weight {
        0..=150 => weight_fc::FC_WEIGHT_THIN,
        151..=250 => weight_fc::FC_WEIGHT_ULTRALIGHT,
        251..=350 => weight_fc::FC_WEIGHT_LIGHT,
        351..=450 => weight_fc::FC_WEIGHT_NORMAL,
        451..=550 => weight_fc::FC_WEIGHT_MEDIUM,
        551..=650 => weight_fc::FC_WEIGHT_DEMIBOLD,
        651..=750 => weight_fc::FC_WEIGHT_BOLD,
        751..=850 => weight_fc::FC_WEIGHT_ULTRABOLD,
        _ => weight_fc::FC_WEIGHT_BLACK,
    }
}

unsafe fn get_bool(pattern: *mut fc::FcPattern, property: &CStr, value: &mut bool) {
    if let Some(matched) = unsafe { get_bool_value(pattern, property) } {
        *value = matched;
    }
}

unsafe fn get_bool_value(pattern: *mut fc::FcPattern, property: &CStr) -> Option<bool> {
    let mut value = 0;
    (unsafe { fc::FcPatternGetBool(pattern, property.as_ptr(), 0, &mut value) }
        == fc::FcResultMatch as u32)
        .then_some(value != 0)
}

unsafe fn get_integer(pattern: *mut fc::FcPattern, property: &CStr, value: &mut i32) {
    let mut matched = 0;
    if unsafe { fc::FcPatternGetInteger(pattern, property.as_ptr(), 0, &mut matched) }
        == fc::FcResultMatch as u32
    {
        *value = matched;
    }
}

struct Pattern(*mut fc::FcPattern);

impl Drop for Pattern {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // The pointer was created by Fontconfig and remains owned until this drop.
            unsafe { fc::FcPatternDestroy(self.0) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{matched_family, render_params};
    use fontconfig_sys as fc;

    #[test]
    fn resolves_generic_families_through_system_fontconfig() {
        for family in ["sans-serif", "serif", "monospace"] {
            assert!(
                matched_family(family).is_some_and(|matched| !matched.is_empty()),
                "Fontconfig did not resolve {family}"
            );
        }
    }

    #[test]
    fn queries_font_rendering_preferences_for_a_face_and_size() {
        let params = render_params("sans-serif", 16.0, 400, false)
            .expect("Fontconfig should return render parameters for sans-serif");

        assert!(params.hinting >= fc::constants::FC_HINT_NONE);
        assert!(params.hinting <= fc::constants::FC_HINT_FULL);
    }
}
