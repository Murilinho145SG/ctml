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

/// One property and value from a CSS rule.
pub type StyleDeclaration = ctml_css::CssDeclaration;
