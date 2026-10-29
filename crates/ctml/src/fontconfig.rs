use std::ffi::{CStr, CString, c_char};

use fontconfig_sys as fc;

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
    use super::matched_family;

    #[test]
    fn resolves_generic_families_through_system_fontconfig() {
        for family in ["sans-serif", "serif", "monospace"] {
            assert!(
                matched_family(family).is_some_and(|matched| !matched.is_empty()),
                "Fontconfig did not resolve {family}"
            );
        }
    }
}
