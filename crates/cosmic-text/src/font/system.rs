use crate::{Attrs, Font, FontMatchAttrs, FontVariation, HashMap, ShapeBuffer};
use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::ops::{Deref, DerefMut};
use fontdb::{FaceInfo, Query, Style};
use skrifa::raw::{ReadError, TableProvider as _};
use skrifa::MetadataProvider;

// re-export fontdb and harfrust
pub use fontdb;
pub use harfrust;

use super::fallback::{Fallback, Fallbacks, MonospaceFallbackInfo, PlatformFallback};

// The fields are used in the derived Ord implementation for sorting fallback candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontMatchKey {
    pub(crate) not_emoji: bool,
    pub(crate) font_weight_diff: u16,
    pub(crate) font_stretch_diff: u16,
    pub(crate) font_style_diff: u8,
    pub(crate) font_weight: u16,
    pub(crate) font_stretch: u16,
    pub(crate) id: fontdb::ID,
    pub(crate) variable_weight_match: bool,
}

impl FontMatchKey {
    fn new(
        attrs: &Attrs,
        face: &FaceInfo,
        db: &fontdb::Database,
        ranges: Option<(Option<(f32, f32)>, Option<(f32, f32)>)>,
    ) -> FontMatchKey {
        // TODO: smarter way of detecting emoji
        let not_emoji = !face.post_script_name.contains("Emoji");
        let font_weight_diff = ranges.and_then(|(weight, _)| weight).map_or_else(
            || attrs.weight.0.abs_diff(face.weight.0),
            |(start, end)| {
                let requested = f32::from(attrs.weight.0);
                if (start..=end).contains(&requested) {
                    0
                } else {
                    (requested - start)
                        .abs()
                        .min((requested - end).abs())
                        .round() as u16
                }
            },
        );

        let variable_weight_match = font_weight_diff != 0
            && if ranges.and_then(|(weight, _)| weight).is_some() {
                true
            } else {
                db.with_face_data(face.id, |font_data, face_index| {
                    let font_ref = skrifa::FontRef::from_index(font_data, face_index).ok()?;
                    let axis = font_ref.axes().get_by_tag(skrifa::Tag::new(b"wght"))?;
                    let w = attrs.weight.0 as f32;
                    Some(w >= axis.min_value() && w <= axis.max_value())
                }) == Some(Some(true))
            };
        let font_weight = face.weight.0;
        let font_stretch_diff = attrs.stretch.to_number().abs_diff(face.stretch.to_number());
        let font_stretch = face.stretch.to_number();
        let mut font_style_diff = match (attrs.style, face.style) {
            (Style::Normal, Style::Normal)
            | (Style::Italic, Style::Italic)
            | (Style::Oblique, Style::Oblique) => 0,
            (Style::Italic, Style::Oblique) | (Style::Oblique, Style::Italic) => 1,
            (Style::Normal, Style::Italic)
            | (Style::Normal, Style::Oblique)
            | (Style::Italic, Style::Normal)
            | (Style::Oblique, Style::Normal) => 2,
        };
        if attrs.style == Style::Oblique {
            if let Some((_, Some((start, end)))) = ranges {
                if !(start..=end).contains(&14.0) {
                    font_style_diff = 2;
                }
            }
        }
        let id = face.id;
        FontMatchKey {
            not_emoji,
            font_weight_diff,
            font_stretch_diff,
            font_style_diff,
            font_weight,
            font_stretch,
            id,
            variable_weight_match,
        }
    }
}

struct FontCachedCodepointSupportInfo {
    supported: Vec<u32>,
    not_supported: Vec<u32>,
}

impl FontCachedCodepointSupportInfo {
    const SUPPORTED_MAX_SZ: usize = 512;
    const NOT_SUPPORTED_MAX_SZ: usize = 1024;

    fn new() -> Self {
        Self {
            supported: Vec::with_capacity(Self::SUPPORTED_MAX_SZ),
            not_supported: Vec::with_capacity(Self::NOT_SUPPORTED_MAX_SZ),
        }
    }

    #[inline(always)]
    fn unknown_has_codepoint(
        &mut self,
        font_codepoints: &[u32],
        codepoint: u32,
        supported_insert_pos: usize,
        not_supported_insert_pos: usize,
    ) -> bool {
        let ret = font_codepoints.contains(&codepoint);
        if ret {
            // don't bother inserting if we are going to truncate the entry away
            if supported_insert_pos != Self::SUPPORTED_MAX_SZ {
                self.supported.insert(supported_insert_pos, codepoint);
                self.supported.truncate(Self::SUPPORTED_MAX_SZ);
            }
        } else {
            // don't bother inserting if we are going to truncate the entry away
            if not_supported_insert_pos != Self::NOT_SUPPORTED_MAX_SZ {
                self.not_supported
                    .insert(not_supported_insert_pos, codepoint);
                self.not_supported.truncate(Self::NOT_SUPPORTED_MAX_SZ);
            }
        }
        ret
    }

    #[inline(always)]
    fn has_codepoint(&mut self, font_codepoints: &[u32], codepoint: u32) -> bool {
        match self.supported.binary_search(&codepoint) {
            Ok(_) => true,
            Err(supported_insert_pos) => match self.not_supported.binary_search(&codepoint) {
                Ok(_) => false,
                Err(not_supported_insert_pos) => self.unknown_has_codepoint(
                    font_codepoints,
                    codepoint,
                    supported_insert_pos,
                    not_supported_insert_pos,
                ),
            },
        }
    }
}

/// Access to the system fonts.
pub struct FontSystem {
    /// The locale of the system.
    locale: String,

    /// The underlying font database.
    db: fontdb::Database,

    /// Cache for loaded fonts from the database.
    font_cache: HashMap<(fontdb::ID, fontdb::Weight, u64), Option<Arc<Font>>>,
    variation_ids: HashMap<Vec<FontVariation>, u64>,
    variations_by_id: HashMap<u64, Vec<FontVariation>>,
    next_variation_id: u64,

    /// Sorted unique ID's of all Monospace fonts in DB
    monospace_font_ids: Vec<fontdb::ID>,

    /// Sorted unique ID's of all Monospace fonts in DB per script.
    /// A font may support multiple scripts of course, so the same ID
    /// may appear in multiple map value vecs.
    per_script_monospace_font_ids: HashMap<[u8; 4], Vec<fontdb::ID>>,

    /// Cache for font codepoint support info
    font_codepoint_support_info_cache: HashMap<fontdb::ID, FontCachedCodepointSupportInfo>,

    /// Cache for font matches.
    font_matches_cache: HashMap<FontMatchAttrs, Arc<Vec<FontMatchKey>>>,
    font_face_ranges: HashMap<fontdb::ID, (Option<(f32, f32)>, Option<(f32, f32)>)>,

    /// Scratch buffer for shaping and laying out.
    pub(crate) shape_buffer: ShapeBuffer,

    /// Buffer for use in `FontFallbackIter`.
    pub(crate) monospace_fallbacks_buffer: BTreeSet<MonospaceFallbackInfo>,

    /// Cache for shaped runs
    #[cfg(feature = "shape-run-cache")]
    pub shape_run_cache: crate::ShapeRunCache,

    /// List of fallbacks
    pub(crate) dyn_fallback: Box<dyn Fallback>,

    /// List of fallbacks
    pub(crate) fallbacks: Fallbacks,
}

impl fmt::Debug for FontSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontSystem")
            .field("locale", &self.locale)
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl FontSystem {
    const FONT_MATCHES_CACHE_SIZE_LIMIT: usize = 256;
    /// Create a new [`FontSystem`], that allows access to any installed system fonts
    ///
    /// # Timing
    ///
    /// This function takes some time to run. On the release build, it can take up to a second,
    /// while debug builds can take up to ten times longer. For this reason, it should only be
    /// called once, and the resulting [`FontSystem`] should be shared.
    pub fn new() -> Self {
        Self::new_with_fonts(core::iter::empty())
    }

    /// Create a new [`FontSystem`] with a pre-specified set of fonts.
    pub fn new_with_fonts(fonts: impl IntoIterator<Item = fontdb::Source>) -> Self {
        let locale = Self::get_locale();
        log::debug!("Locale: {locale}");

        let mut db = fontdb::Database::new();

        Self::load_fonts(&mut db, fonts.into_iter());

        //TODO: configurable default fonts
        db.set_monospace_family("Noto Sans Mono");
        db.set_sans_serif_family("Open Sans");
        db.set_serif_family("DejaVu Serif");

        Self::new_with_locale_and_db_and_fallback(locale, db, PlatformFallback)
    }

    /// Create a new [`FontSystem`] with a pre-specified locale, font database and font fallback list.
    pub fn new_with_locale_and_db_and_fallback(
        locale: String,
        db: fontdb::Database,
        impl_fallback: impl Fallback + 'static,
    ) -> Self {
        let mut monospace_font_ids = db
            .faces()
            .filter(|face_info| {
                face_info.monospaced && !face_info.post_script_name.contains("Emoji")
            })
            .map(|face_info| face_info.id)
            .collect::<Vec<_>>();
        monospace_font_ids.sort();

        let mut per_script_monospace_font_ids: HashMap<[u8; 4], BTreeSet<fontdb::ID>> =
            HashMap::default();

        if cfg!(feature = "monospace_fallback") {
            for &id in &monospace_font_ids {
                db.with_face_data(id, |font_data, face_index| {
                    let face = skrifa::FontRef::from_index(font_data, face_index)?;
                    for script in face
                        .gpos()?
                        .script_list()?
                        .script_records()
                        .iter()
                        .chain(face.gsub()?.script_list()?.script_records().iter())
                    {
                        per_script_monospace_font_ids
                            .entry(script.script_tag().into_bytes())
                            .or_default()
                            .insert(id);
                    }
                    Ok::<_, ReadError>(())
                });
            }
        }

        let per_script_monospace_font_ids = per_script_monospace_font_ids
            .into_iter()
            .map(|(k, v)| (k, Vec::from_iter(v)))
            .collect();

        let fallbacks = Fallbacks::new(&impl_fallback, &[], &locale);

        Self {
            locale,
            db,
            monospace_font_ids,
            per_script_monospace_font_ids,
            font_cache: HashMap::default(),
            variation_ids: HashMap::default(),
            variations_by_id: HashMap::default(),
            next_variation_id: 1,
            font_matches_cache: HashMap::default(),
            font_face_ranges: HashMap::default(),
            font_codepoint_support_info_cache: HashMap::default(),
            monospace_fallbacks_buffer: BTreeSet::default(),
            #[cfg(feature = "shape-run-cache")]
            shape_run_cache: crate::ShapeRunCache::default(),
            shape_buffer: ShapeBuffer::default(),
            dyn_fallback: Box::new(impl_fallback),
            fallbacks,
        }
    }

    /// Create a new [`FontSystem`] with a pre-specified locale and font database.
    pub fn new_with_locale_and_db(locale: String, db: fontdb::Database) -> Self {
        Self::new_with_locale_and_db_and_fallback(locale, db, PlatformFallback)
    }

    /// Get the locale.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Get the database.
    pub const fn db(&self) -> &fontdb::Database {
        &self.db
    }

    /// Get a mutable reference to the database.
    pub fn db_mut(&mut self) -> &mut fontdb::Database {
        self.font_matches_cache.clear();
        &mut self.db
    }

    /// Associates CSS @font-face descriptor ranges with a registered face.
    pub fn set_font_face_ranges(
        &mut self,
        id: fontdb::ID,
        weight_range: Option<(f32, f32)>,
        oblique_range: Option<(f32, f32)>,
    ) {
        self.font_face_ranges
            .insert(id, (weight_range, oblique_range));
        self.font_matches_cache.clear();
    }

    /// Consume this [`FontSystem`] and return the locale and database.
    pub fn into_locale_and_db(self) -> (String, fontdb::Database) {
        (self.locale, self.db)
    }

    /// Get a font by its ID and weight.
    pub fn get_font(&mut self, id: fontdb::ID, weight: fontdb::Weight) -> Option<Arc<Font>> {
        self.get_font_with_variations(id, weight, &[])
    }

    /// Get a font instance with additional OpenType variation settings.
    pub fn get_font_with_variations(
        &mut self,
        id: fontdb::ID,
        weight: fontdb::Weight,
        variations: &[FontVariation],
    ) -> Option<Arc<Font>> {
        let variation_key = if variations.is_empty() {
            0
        } else if let Some(key) = self.variation_ids.get(variations) {
            *key
        } else {
            let key = self.next_variation_id;
            self.next_variation_id = self.next_variation_id.wrapping_add(1).max(1);
            let variations = variations.to_vec();
            self.variation_ids.insert(variations.clone(), key);
            self.variations_by_id.insert(key, variations);
            key
        };
        self.get_font_with_variation_key(id, weight, variation_key)
    }

    /// Get a font by the variation key carried by shaped glyphs.
    pub fn get_font_with_variation_key(
        &mut self,
        id: fontdb::ID,
        weight: fontdb::Weight,
        variation_key: u64,
    ) -> Option<Arc<Font>> {
        let mut variations = self
            .variations_by_id
            .get(&variation_key)
            .cloned()
            .unwrap_or_default();
        if !variations
            .iter()
            .any(|variation| variation.tag() == *b"wght")
        {
            if let Some(Some((min, max))) = self.font_face_ranges.get(&id).map(|(weight, _)| weight)
            {
                let selected_weight = f32::from(weight.0).clamp(*min, *max);
                if selected_weight != f32::from(weight.0) {
                    variations.push(FontVariation::new(*b"wght", selected_weight));
                }
            }
        }
        self.font_cache
            .entry((id, weight, variation_key))
            .or_insert_with(|| {
                #[cfg(feature = "std")]
                unsafe {
                    self.db.make_shared_face_data(id);
                }
                if let Some(font) = Font::new(&self.db, id, weight, &variations, variation_key) {
                    Some(Arc::new(font))
                } else {
                    log::warn!(
                        "failed to load font '{}'",
                        self.db.face(id)?.post_script_name
                    );
                    None
                }
            })
            .clone()
    }

    pub fn is_monospace(&self, id: fontdb::ID) -> bool {
        self.monospace_font_ids.binary_search(&id).is_ok()
    }

    pub fn get_monospace_ids_for_scripts(
        &self,
        scripts: impl Iterator<Item = [u8; 4]>,
    ) -> Vec<fontdb::ID> {
        let mut ret = scripts
            .filter_map(|script| self.per_script_monospace_font_ids.get(&script))
            .flat_map(|ids| ids.iter().copied())
            .collect::<Vec<_>>();
        ret.sort();
        ret.dedup();
        ret
    }

    #[inline(always)]
    pub fn get_font_supported_codepoints_in_word(
        &mut self,
        id: fontdb::ID,
        weight: fontdb::Weight,
        word: &str,
    ) -> Option<usize> {
        self.get_font(id, weight).map(|font| {
            let code_points = font.unicode_codepoints();
            let cache = self
                .font_codepoint_support_info_cache
                .entry(id)
                .or_insert_with(FontCachedCodepointSupportInfo::new);
            word.chars()
                .filter(|ch| cache.has_codepoint(code_points, u32::from(*ch)))
                .count()
        })
    }

    pub fn get_font_matches(&mut self, attrs: &Attrs<'_>) -> Arc<Vec<FontMatchKey>> {
        // Clear the cache first if it reached the size limit
        if self.font_matches_cache.len() >= Self::FONT_MATCHES_CACHE_SIZE_LIMIT {
            log::trace!("clear font mache cache");
            self.font_matches_cache.clear();
        }

        self.font_matches_cache
            //TODO: do not create AttrsOwned unless entry does not already exist
            .entry(attrs.into())
            .or_insert_with(|| {
                #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                let now = std::time::Instant::now();

                let face_ranges = &self.font_face_ranges;
                let mut font_match_keys = self
                    .db
                    .faces()
                    .map(|face| {
                        FontMatchKey::new(attrs, face, &self.db, face_ranges.get(&face.id).copied())
                    })
                    .collect::<Vec<_>>();

                // Sort so we get the keys with weight_offset=0 first
                font_match_keys.sort();

                // db.query is better than above, but returns just one font
                let query = Query {
                    families: &[attrs.family],
                    weight: attrs.weight,
                    stretch: attrs.stretch,
                    style: attrs.style,
                };

                if let Some(id) = self.db.query(&query) {
                    if !face_ranges.contains_key(&id) {
                        if let Some(i) = font_match_keys
                            .iter()
                            .enumerate()
                            .find(|(_i, key)| key.id == id)
                            .map(|(i, _)| i)
                        {
                            // if exists move to front
                            let match_key = font_match_keys.remove(i);
                            font_match_keys.insert(0, match_key);
                        } else if let Some(face) = self.db.face(id) {
                            // else insert in front
                            let match_key = FontMatchKey::new(attrs, face, &self.db, None);
                            font_match_keys.insert(0, match_key);
                        } else {
                            log::error!("Could not get face from db, that should've been there.");
                        }
                    }
                }

                #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                {
                    let elapsed = now.elapsed();
                    log::debug!("font matches for {attrs:?} in {elapsed:?}");
                }

                Arc::new(font_match_keys)
            })
            .clone()
    }

    #[cfg(feature = "std")]
    fn get_locale() -> String {
        sys_locale::get_locale().unwrap_or_else(|| {
            log::warn!("failed to get system locale, falling back to en-US");
            String::from("en-US")
        })
    }

    #[cfg(not(feature = "std"))]
    fn get_locale() -> String {
        String::from("en-US")
    }

    #[cfg(feature = "std")]
    fn load_fonts(db: &mut fontdb::Database, fonts: impl Iterator<Item = fontdb::Source>) {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();

        db.load_system_fonts();

        for source in fonts {
            db.load_font_source(source);
        }

        #[cfg(not(target_arch = "wasm32"))]
        log::debug!(
            "Parsed {} font faces in {}ms.",
            db.len(),
            now.elapsed().as_millis()
        );
    }

    #[cfg(not(feature = "std"))]
    fn load_fonts(db: &mut fontdb::Database, fonts: impl Iterator<Item = fontdb::Source>) {
        for source in fonts {
            db.load_font_source(source);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FontMatchKey, FontSystem};
    use crate::{Attrs, Family};
    use fontdb::{Style, Weight};

    #[test]
    fn font_face_ranges_constrain_weight_and_default_oblique_angle_matching() {
        let mut font_system = FontSystem::new();
        let mut face = font_system
            .db()
            .faces()
            .next()
            .expect("system should have at least one font")
            .clone();
        face.style = Style::Oblique;
        let family_name = face
            .families
            .first()
            .expect("font should have a family")
            .0
            .as_str();
        let ranges = Some((Some((300.5, 700.75)), Some((10.0, 20.0))));

        let inside = Attrs::new()
            .family(Family::Name(family_name))
            .weight(Weight(500))
            .style(Style::Oblique);
        let key = FontMatchKey::new(&inside, &face, font_system.db(), ranges);
        assert_eq!(key.font_weight_diff, 0);
        assert_eq!(key.font_style_diff, 0);

        let outside = Attrs::new()
            .family(Family::Name(family_name))
            .weight(Weight(200))
            .style(Style::Oblique);
        let key = FontMatchKey::new(&outside, &face, font_system.db(), ranges);
        assert_eq!(key.font_weight_diff, 101);
        assert_eq!(key.font_style_diff, 0);
        let outside_style = FontMatchKey::new(
            &inside,
            &face,
            font_system.db(),
            Some((Some((300.5, 700.75)), Some((20.0, 30.0)))),
        );
        assert_eq!(outside_style.font_style_diff, 2);

        font_system.set_font_face_ranges(face.id, Some((300.5, 700.75)), None);
        let font = font_system
            .get_font_with_variations(face.id, Weight(800), &[])
            .expect("registered font should load");
        assert_eq!(
            font.variations()
                .iter()
                .find(|variation| variation.tag() == *b"wght")
                .map(|variation| variation.value()),
            Some(700.75)
        );
    }
}

/// A value borrowed together with an [`FontSystem`]
#[derive(Debug)]
pub struct BorrowedWithFontSystem<'a, T> {
    pub(crate) inner: &'a mut T,
    pub(crate) font_system: &'a mut FontSystem,
}

impl<T> Deref for BorrowedWithFontSystem<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T> DerefMut for BorrowedWithFontSystem<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
