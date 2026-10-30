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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BorderImageWidth {
    Number(f32),
    Length(f32),
    Percentage(f32),
    Auto,
}

impl Default for BorderImageSlice {
    fn default() -> Self {
        Self {
            value: 100.0,
            percentage: true,
        }
    }
}
