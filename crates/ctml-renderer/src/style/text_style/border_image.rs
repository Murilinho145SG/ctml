#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BorderImageSlice {
    pub(crate) value: f32,
    pub(crate) percentage: bool,
}

impl Default for BorderImageSlice {
    fn default() -> Self {
        Self {
            value: 100.0,
            percentage: true,
        }
    }
}
