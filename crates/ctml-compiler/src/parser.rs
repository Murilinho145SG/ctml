use std::collections::HashSet;

use crate::{Error, Lexer, Position, Result, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub headers: Vec<Node>,
    pub body: Vec<Node>,
}

impl Document {
    pub fn application(&self) -> Option<&Element> {
        self.headers.iter().find_map(|node| match node {
            Node::Element(element) if element.name == "application" => Some(element),
            _ => None,
        })
    }

    pub fn stylesheet(&self) -> Option<&Element> {
        self.stylesheets().next()
    }

    pub fn stylesheets(&self) -> impl Iterator<Item = &Element> {
        self.headers.iter().filter_map(|node| match node {
            Node::Element(element) if element.name == "stylesheet" => Some(element),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
}

impl Element {
    pub fn attribute(&self, name: &str) -> Option<&AttributeValue> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name == name)
            .map(|attribute| &attribute.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    Boolean,
    String(String),
}

impl AttributeValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Boolean => None,
            Self::String(value) => Some(value),
        }
    }
}

pub fn parse(source: &str) -> Result<Document> {
    let tokens = Lexer::new(source).tokenize()?;
    Parser { tokens, index: 0 }.parse_document()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn parse_document(mut self) -> Result<Document> {
        self.skip_whitespace_text();
        let headers = self.parse_element()?;
        if headers.name != "headers" {
            return Err(self.syntax_at(
                format!("expected <headers>, found <{}>", headers.name),
                self.previous_position(),
            ));
        }

        self.skip_whitespace_text();
        let body = self.parse_element()?;
        if body.name != "body" {
            return Err(self.syntax_at(
                format!("expected <body>, found <{}>", body.name),
                self.previous_position(),
            ));
        }

        self.skip_whitespace_text();
        if !matches!(self.peek().kind, TokenKind::Eof) {
            return Err(self.syntax("unexpected content after </body>"));
        }

        Ok(Document {
            headers: headers.children,
            body: body.children,
        })
    }

    fn parse_element(&mut self) -> Result<Element> {
        self.expect_start_tag_open()?;
        let name = self.expect_tag_name()?;
        let mut attributes = Vec::new();
        let mut attribute_names = HashSet::new();

        loop {
            match self.peek().kind.clone() {
                TokenKind::AttributeName(attribute_name) => {
                    let position = self.advance().span.start;
                    if !attribute_names.insert(attribute_name.clone()) {
                        return Err(self.syntax_at(
                            format!("duplicate attribute '{attribute_name}'"),
                            position,
                        ));
                    }

                    let value = if matches!(self.peek().kind, TokenKind::Equals) {
                        self.advance();
                        match self.advance().kind {
                            TokenKind::StringLiteral(value) => AttributeValue::String(value),
                            _ => {
                                return Err(self.syntax_at(
                                    "expected a quoted attribute value".to_owned(),
                                    self.previous_position(),
                                ));
                            }
                        }
                    } else {
                        AttributeValue::Boolean
                    };

                    attributes.push(Attribute {
                        name: attribute_name,
                        value,
                    });
                }
                TokenKind::TagEnd => {
                    self.advance();
                    let children = self.parse_children(&name)?;
                    return Ok(Element {
                        name,
                        attributes,
                        children,
                    });
                }
                TokenKind::SelfClosingTagEnd => {
                    self.advance();
                    return Ok(Element {
                        name,
                        attributes,
                        children: Vec::new(),
                    });
                }
                _ => return Err(self.syntax("expected an attribute or the end of a tag")),
            }
        }
    }

    fn parse_children(&mut self, parent_name: &str) -> Result<Vec<Node>> {
        let mut children = Vec::new();
        loop {
            match self.peek().kind.clone() {
                TokenKind::Text(text) => {
                    self.advance();
                    children.push(Node::Text(text));
                }
                TokenKind::StartTagOpen => children.push(Node::Element(self.parse_element()?)),
                TokenKind::EndTagOpen => {
                    self.advance();
                    let closing_name = self.expect_tag_name()?;
                    if closing_name != parent_name {
                        return Err(self.syntax_at(
                            format!("closing tag </{closing_name}> does not match <{parent_name}>"),
                            self.previous_position(),
                        ));
                    }
                    self.expect_tag_end()?;
                    return Ok(children);
                }
                TokenKind::Eof => {
                    return Err(self.syntax_at(
                        format!("missing closing tag </{parent_name}>"),
                        self.peek().span.start,
                    ));
                }
                _ => return Err(self.syntax("expected text, an element, or a closing tag")),
            }
        }
    }

    fn expect_start_tag_open(&mut self) -> Result<()> {
        if matches!(self.peek().kind, TokenKind::StartTagOpen) {
            self.advance();
            Ok(())
        } else {
            Err(self.syntax("expected an opening tag"))
        }
    }

    fn expect_tag_name(&mut self) -> Result<String> {
        match self.advance().kind {
            TokenKind::TagName(name) => Ok(name),
            _ => Err(self.syntax_at("expected a tag name".to_owned(), self.previous_position())),
        }
    }

    fn expect_tag_end(&mut self) -> Result<()> {
        if matches!(self.peek().kind, TokenKind::TagEnd) {
            self.advance();
            Ok(())
        } else {
            Err(self.syntax("expected '>' after closing tag name"))
        }
    }

    fn skip_whitespace_text(&mut self) {
        while let TokenKind::Text(text) = &self.peek().kind {
            if !text.chars().all(char::is_whitespace) {
                break;
            }
            self.advance();
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        if !matches!(token.kind, TokenKind::Eof) {
            self.index += 1;
        }
        token
    }

    fn previous_position(&self) -> Position {
        self.tokens[self.index.saturating_sub(1)].span.start
    }

    fn syntax(&self, message: &str) -> Error {
        self.syntax_at(message.to_owned(), self.peek().span.start)
    }

    fn syntax_at(&self, message: String, position: Position) -> Error {
        Error::Syntax {
            message,
            line: position.line,
            column: position.column,
        }
    }
}
