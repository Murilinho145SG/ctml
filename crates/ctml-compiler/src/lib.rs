mod error;
mod lexer;
mod parser;

pub use ctml_css::{
    CssDeclaration, CssFontFace, CssRule, Stylesheet, load_css_resource, parse_stylesheet,
    parse_stylesheet_file,
};
pub use error::{Error, Result};
pub use lexer::{Lexer, Position, Span, Token, TokenKind};
pub use parser::{Attribute, AttributeValue, Document, Element, Node, parse};

#[cfg(test)]
mod tests;
