mod css_style;
mod font_family;
mod font_metrics;
#[cfg(target_os = "linux")]
mod fontconfig;
mod page;
mod renderer;
mod style;
mod ui;
mod window;

pub use ui::{StyleDeclaration, StyleRule, UiNode};
/// A local font loaded from a stylesheet `@font-face` rule.
#[derive(Clone, Debug)]
pub struct FontFace {
    pub family: String,
    pub bytes: Vec<u8>,
    pub weight: Option<u16>,
    pub style: Option<String>,
    pub stretch: Option<String>,
}
pub use window::{Window, WindowConfig, WindowError};
