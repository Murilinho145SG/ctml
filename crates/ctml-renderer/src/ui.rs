use std::sync::Arc;

/// A small, compiled representation of content rendered by CTML.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiNode {
    /// Text content in document order.
    Text(String),
    /// An element and its child content.
    Element {
        tag: String,
        class: Option<String>,
        id: Option<String>,
        /// Element attributes as (name, string value). Boolean attributes use `None`.
        attributes: Vec<(String, Option<String>)>,
        children: Vec<UiNode>,
    },
}

/// A parsed CSS rule passed to CTML's native style resolver.
pub type StyleRule = ctml_css::CssRule;

/// Handles a named CTML action and may update the body before it is rendered again.
pub type UiActionHandler = Arc<dyn Fn(&str, &mut Vec<UiNode>) + Send + Sync + 'static>;

/// Handles text changes from an editable `<input>` and may update the body before it is rendered again.
pub type UiInputHandler = Arc<dyn Fn(&str, &str, &mut Vec<UiNode>) + Send + Sync + 'static>;

/// One property and value from a CSS rule.
pub type StyleDeclaration = ctml_css::CssDeclaration;
