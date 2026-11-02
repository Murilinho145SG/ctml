use crate::UiNode;

use super::PageRasterizer;

impl PageRasterizer {
    pub(crate) fn replace_nodes(&mut self, nodes: &[UiNode]) {
        let mut updated = Self::new_with_fonts_color_profiles_and_keyframes(
            nodes,
            &self.authored_stylesheet,
            &self.keyframes,
            self.animations_enabled,
            &self.fonts,
            &self.color_profiles,
        );
        if let Some((width, height)) = self.layout_size {
            updated.prepare_layout(width, height);
        }
        *self = updated;
    }
}
