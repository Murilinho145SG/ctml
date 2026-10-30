use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BackgroundImage {
    None,
    Encoded(Arc<[u8]>),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    ConicGradient(ConicGradient),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LinearGradient {
    pub(crate) angle_degrees: f32,
    pub(crate) stops: Vec<([u8; 4], f32)>,
    pub(crate) repeating: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum RadialGradientShape {
    Circle,
    #[default]
    Ellipse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum RadialGradientExtent {
    ClosestSide,
    FarthestSide,
    ClosestCorner,
    #[default]
    FarthestCorner,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RadialGradient {
    pub(crate) shape: RadialGradientShape,
    pub(crate) extent: RadialGradientExtent,
    pub(crate) center_x_percent: f32,
    pub(crate) center_y_percent: f32,
    pub(crate) stops: Vec<([u8; 4], f32)>,
    pub(crate) repeating: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConicGradient {
    pub(crate) from_degrees: f32,
    pub(crate) center_x_percent: f32,
    pub(crate) center_y_percent: f32,
    pub(crate) stops: Vec<([u8; 4], f32)>,
    pub(crate) repeating: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct BackgroundRepeat {
    pub(crate) x: bool,
    pub(crate) y: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BackgroundPosition {
    pub(crate) x_percent: f32,
    pub(crate) x_offset: f32,
    pub(crate) y_percent: f32,
    pub(crate) y_offset: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BackgroundSizeAxis {
    pub(crate) value: f32,
    pub(crate) percentage: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum BackgroundFit {
    #[default]
    Auto,
    Cover,
    Contain,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum BackgroundAttachment {
    #[default]
    Scroll,
    Fixed,
    Local,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum BackgroundBox {
    #[default]
    BorderBox,
    PaddingBox,
    ContentBox,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BackgroundSize {
    pub(crate) width: Option<BackgroundSizeAxis>,
    pub(crate) height: Option<BackgroundSizeAxis>,
    pub(crate) fit: BackgroundFit,
}

impl BackgroundRepeat {
    pub(crate) const REPEAT: Self = Self { x: true, y: true };
    pub(crate) const NO_REPEAT: Self = Self { x: false, y: false };
    pub(crate) const REPEAT_X: Self = Self { x: true, y: false };
    pub(crate) const REPEAT_Y: Self = Self { x: false, y: true };
}
