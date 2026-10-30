mod error;
mod lexer;
mod parser;

pub use ctml_css::{
    CssDeclaration, CssFontFace, CssFontSource, CssRule, Stylesheet, load_css_resource,
    load_local_resource, parse_font_face_sources, parse_stylesheet, parse_stylesheet_file,
};
pub use error::{Error, Result};
pub use lexer::{Lexer, Position, Span, Token, TokenKind};
pub use parser::{Attribute, AttributeValue, Document, Element, Node, parse};

#[cfg(test)]
mod tests;
