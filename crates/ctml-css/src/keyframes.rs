use cssparser::{Delimiter, Parser, ParserInput, Token};

use super::{CssDeclaration, parse_declaration_block};

/// A parsed `@keyframes` rule and its keyframe blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframes {
    pub name: String,
    pub frames: Vec<CssKeyframe>,
}

/// A keyframe block with one or more normalized offsets in the inclusive range `0..=1`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssKeyframe {
    pub offsets: Vec<f32>,
    pub declarations: Vec<CssDeclaration>,
}

pub(super) fn parse_keyframes_name(prelude: &str) -> Option<String> {
    let mut source = ParserInput::new(prelude);
    let mut input = Parser::new(&mut source);
    match input.next().ok()? {
        Token::AtKeyword(name) if name.eq_ignore_ascii_case("keyframes") => {}
        _ => return None,
    }
    let name = input.expect_ident_cloned().ok()?.to_string();
    input.is_exhausted().then_some(name)
}

fn parse_keyframe_offsets(value: &str) -> Option<Vec<f32>> {
    let mut offsets = Vec::new();
    for selector in value.split(',') {
        let selector = selector.trim();
        let offset = if selector.eq_ignore_ascii_case("from") {
            0.0
        } else if selector.eq_ignore_ascii_case("to") {
            1.0
        } else {
            let percentage = crate::parse_css_percentage(selector)?;
            if !(0.0..=100.0).contains(&percentage) {
                return None;
            }
            percentage / 100.0
        };
        offsets.push(offset);
    }
    (!offsets.is_empty()).then_some(offsets)
}

pub(super) fn parse_keyframes_block<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Vec<CssKeyframe>, cssparser::ParseError<'i, ()>> {
    let mut frames = Vec::new();
    while !input.is_exhausted() {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }
        let prelude = input.parse_until_before::<_, _, ()>(
            Delimiter::CurlyBracketBlock | Delimiter::Semicolon,
            |prelude| {
                let start = prelude.position();
                while prelude.next_including_whitespace().is_ok() {}
                Ok(prelude.slice(start..prelude.position()).trim().to_owned())
            },
        )?;
        if input.try_parse(|input| input.expect_semicolon()).is_ok() {
            continue;
        }
        if input.expect_curly_bracket_block().is_err() {
            let _ = input.next_including_whitespace();
            continue;
        }
        let declarations = input.parse_nested_block(parse_declaration_block)?;
        if let Some(offsets) = parse_keyframe_offsets(&prelude) {
            frames.push(CssKeyframe {
                offsets,
                declarations,
            });
        }
    }
    Ok(frames)
}
