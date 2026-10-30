use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BackgroundImage {
    Encoded(Arc<[u8]>),
    LinearGradient(LinearGradient),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LinearGradient {
    pub(crate) angle_degrees: f32,
    pub(crate) stops: Vec<([u8; 4], f32)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct BackgroundRepeat {
    pub(crate) x: bool,
    pub(crate) y: bool,
}

impl BackgroundRepeat {
    pub(crate) const REPEAT: Self = Self { x: true, y: true };
    pub(crate) const NO_REPEAT: Self = Self { x: false, y: false };
    pub(crate) const REPEAT_X: Self = Self { x: true, y: false };
    pub(crate) const REPEAT_Y: Self = Self { x: false, y: true };
}
