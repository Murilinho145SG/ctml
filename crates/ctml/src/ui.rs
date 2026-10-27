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
        children: Vec<UiNode>,
    },
}

/// A parsed CSS rule passed to CTML's native style resolver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleRule {
    pub selectors: Vec<String>,
    pub declarations: Vec<StyleDeclaration>,
}

/// One property and value from a CSS rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleDeclaration {
    pub property: String,
    pub value: String,
}
