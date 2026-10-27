use std::{iter::Peekable, str::CharIndices};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Zero-based UTF-8 byte offset.
    pub offset: usize,
    /// One-based line number.
    pub line: usize,
    /// One-based character column.
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    /// End position is exclusive.
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    StartTagOpen,
    EndTagOpen,
    TagName(String),
    AttributeName(String),
    Equals,
    StringLiteral(String),
    TagEnd,
    SelfClosingTagEnd,
    Text(String),
    Eof,
}

pub struct Lexer<'source> {
    characters: Peekable<CharIndices<'source>>,
    position: Position,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            characters: source.char_indices().peekable(),
            position: Position {
                offset: 0,
                line: 1,
                column: 1,
            },
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while let Some(character) = self.peek_char() {
            let token = if character == '<' {
                tokens.extend(self.scan_tag()?);
                continue;
            } else {
                self.scan_text()?
            };
            tokens.push(token);
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                start: self.position,
                end: self.position,
            },
        });
        Ok(tokens)
    }

    fn scan_text(&mut self) -> Result<Token> {
        let start = self.position;
        let mut value = String::new();

        while let Some(character) = self.peek_char() {
            if character == '<' {
                break;
            }
            value.push(self.next_char().expect("peeked character must exist"));
        }

        Ok(self.token(TokenKind::Text(value), start))
    }

    fn scan_tag(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let open_start = self.position;
        self.next_char();

        let is_end_tag = self.peek_char() == Some('/');
        if is_end_tag {
            self.next_char();
            tokens.push(self.token(TokenKind::EndTagOpen, open_start));
        } else {
            tokens.push(self.token(TokenKind::StartTagOpen, open_start));
        }

        let name_start = self.position;
        let name = self.scan_name("tag name")?;
        tokens.push(self.token(TokenKind::TagName(name), name_start));

        let mut whitespace_after_boolean_attribute = false;
        loop {
            let had_whitespace =
                std::mem::take(&mut whitespace_after_boolean_attribute) || self.skip_whitespace();
            match self.peek_char() {
                Some('>') => {
                    let end_start = self.position;
                    self.next_char();
                    tokens.push(self.token(TokenKind::TagEnd, end_start));
                    return Ok(tokens);
                }
                Some('/') if !is_end_tag => {
                    let end_start = self.position;
                    self.next_char();
                    if self.peek_char() != Some('>') {
                        return Err(self.unexpected("'>' after '/'"));
                    }
                    self.next_char();
                    tokens.push(self.token(TokenKind::SelfClosingTagEnd, end_start));
                    return Ok(tokens);
                }
                Some(_) if is_end_tag => return Err(self.unexpected("'>' after end tag name")),
                Some(_) => {
                    if !had_whitespace {
                        return Err(Error::ExpectedWhitespace {
                            line: self.position.line,
                            column: self.position.column,
                        });
                    }
                    whitespace_after_boolean_attribute = self.scan_attribute(&mut tokens)?;
                }
                None => return Err(self.unexpected_eof("'>'")),
            }
        }
    }

    fn scan_attribute(&mut self, tokens: &mut Vec<Token>) -> Result<bool> {
        let name_start = self.position;
        let name = self.scan_name("attribute name")?;
        tokens.push(self.token(TokenKind::AttributeName(name), name_start));

        let whitespace_after_name = self.skip_whitespace();
        if self.peek_char() != Some('=') {
            return Ok(whitespace_after_name);
        }

        let equals_start = self.position;
        self.next_char();
        tokens.push(self.token(TokenKind::Equals, equals_start));
        self.skip_whitespace();

        if self.peek_char() != Some('"') {
            return Err(self.unexpected("a double-quoted attribute value"));
        }
        let value_start = self.position;
        let value = self.scan_string()?;
        tokens.push(self.token(TokenKind::StringLiteral(value), value_start));
        Ok(false)
    }

    fn scan_string(&mut self) -> Result<String> {
        let opening = self.position;
        self.next_char();
        let mut value = String::new();

        loop {
            match self.next_char() {
                Some('"') => return Ok(value),
                Some('\\') => match self.next_char() {
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some(character) => {
                        return Err(Error::InvalidEscape {
                            character,
                            line: self.position.line,
                            column: self.position.column.saturating_sub(1),
                        });
                    }
                    None => return Err(self.unterminated_string(opening)),
                },
                Some(character) => value.push(character),
                None => return Err(self.unterminated_string(opening)),
            }
        }
    }

    fn scan_name(&mut self, expected: &'static str) -> Result<String> {
        let Some(first) = self.peek_char() else {
            return Err(self.unexpected_eof(expected));
        };
        if !is_name_start(first) {
            return Err(self.unexpected(expected));
        }

        let mut name = String::new();
        name.push(self.next_char().expect("peeked character must exist"));
        while self.peek_char().is_some_and(is_name_continue) {
            name.push(self.next_char().expect("peeked character must exist"));
        }
        Ok(name)
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut skipped = false;
        while self
            .peek_char()
            .is_some_and(|character| character.is_ascii_whitespace())
        {
            self.next_char();
            skipped = true;
        }
        skipped
    }

    fn peek_char(&mut self) -> Option<char> {
        self.characters.peek().map(|(_, character)| *character)
    }

    fn next_char(&mut self) -> Option<char> {
        let (offset, character) = self.characters.next()?;
        self.position.offset = offset + character.len_utf8();
        match character {
            '\n' => {
                self.position.line += 1;
                self.position.column = 1;
            }
            '\r' if self.peek_char() != Some('\n') => {
                self.position.line += 1;
                self.position.column = 1;
            }
            _ => self.position.column += 1,
        }
        Some(character)
    }

    fn token(&self, kind: TokenKind, start: Position) -> Token {
        Token {
            kind,
            span: Span {
                start,
                end: self.position,
            },
        }
    }

    fn unexpected(&mut self, expected: &'static str) -> Error {
        match self.peek_char() {
            Some(character) => Error::UnexpectedCharacter {
                character,
                expected,
                line: self.position.line,
                column: self.position.column,
            },
            None => self.unexpected_eof(expected),
        }
    }

    fn unexpected_eof(&self, expected: &'static str) -> Error {
        Error::UnexpectedEof {
            expected,
            line: self.position.line,
            column: self.position.column,
        }
    }

    fn unterminated_string(&self, start: Position) -> Error {
        Error::UnterminatedString {
            line: start.line,
            column: start.column,
        }
    }
}

fn is_name_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_name_continue(character: char) -> bool {
    is_name_start(character) || character.is_ascii_digit() || character == '-'
}
