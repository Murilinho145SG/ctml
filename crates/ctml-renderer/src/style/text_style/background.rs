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
