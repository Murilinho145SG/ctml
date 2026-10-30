#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BorderImageSlice {
    pub(crate) value: f32,
    pub(crate) percentage: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum BorderImageRepeat {
    #[default]
    Stretch,
    Repeat,
    Round,
    Space,
}

impl Default for BorderImageSlice {
    fn default() -> Self {
        Self {
            value: 100.0,
            percentage: true,
        }
    }
}
