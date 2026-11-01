use std::{cell::RefCell, collections::VecDeque, sync::Arc};

use cosmic_text::{FontSystem, fontdb};
use skia_safe::Typeface;

// Keep font identities stable across reflows so Skia can reuse glyph strikes.
// Source identity also prevents IDs from separate font databases from colliding.
const MAX_TYPEFACES: usize = 32;
thread_local! {
    static TYPEFACES: RefCell<VecDeque<(fontdb::Source, u32, Typeface)>> = RefCell::default();
}

pub(crate) fn load(font_system: &FontSystem, id: fontdb::ID) -> Option<Typeface> {
    let face = font_system.db().face(id)?;
    TYPEFACES.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(position) = cache.iter().position(|(source, index, _)| {
            *index == face.index && same_source(source, &face.source)
        }) {
            let entry = cache.remove(position)?;
            let typeface = entry.2.clone();
            cache.push_back(entry);
            return Some(typeface);
        }
        let typeface = font_system.db().with_face_data(id, |data, index| {
            crate::font_metrics::shared_font_manager().new_from_bytes(data, Some(index))
        })??;
        if cache.len() == MAX_TYPEFACES {
            cache.pop_front();
        }
        cache.push_back((face.source.clone(), face.index, typeface.clone()));
        Some(typeface)
    })
}

fn same_source(left: &fontdb::Source, right: &fontdb::Source) -> bool {
    use fontdb::Source;
    match (left, right) {
        (Source::Binary(left), Source::Binary(right)) => Arc::ptr_eq(left, right),
        (
            Source::File(left) | Source::SharedFile(left, _),
            Source::File(right) | Source::SharedFile(right, _),
        ) => left == right,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_typefaces_across_layout_passes() {
        let system = FontSystem::new();
        let id = system.db().faces().next().expect("a system font").id;
        let first = load(&system, id).expect("a readable font");
        let second = load(&system, id).expect("a cached font");
        assert_eq!(first.unique_id(), second.unique_id());
        for face in system.db().faces().take(MAX_TYPEFACES * 2) {
            load(&system, face.id);
        }
        TYPEFACES.with(|cache| assert!(cache.borrow().len() <= MAX_TYPEFACES));
    }
}
