use cssparser::{Parser, ParserInput, Token};

pub(super) fn parse_missing_rgb_components(value: &str) -> [bool; 4] {
    parse_missing_components(value, 0)
}

fn parse_missing_components(value: &str, depth: usize) -> [bool; 4] {
    if depth > 32 {
        return [false; 4];
    }
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let Ok(function) = parser.expect_function() else {
        return [false; 4];
    };
    let function = function.to_ascii_lowercase();
    let missing = match function.as_str() {
        "color" => parser
            .parse_nested_block(parse_color_components)
            .unwrap_or([false; 4]),
        "color-mix" => parser
            .parse_nested_block(|input| parse_mix_components(input, depth + 1))
            .unwrap_or([false; 4]),
        _ => [false; 4],
    };
    if parser.is_exhausted() {
        missing
    } else {
        [false; 4]
    }
}

fn parse_color_components<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<[bool; 4], cssparser::ParseError<'i, ()>> {
    let profile = input.expect_ident_cloned()?.to_ascii_lowercase();
    let rgb_profile = is_rgb_profile(&profile);
    let mut missing = [false; 4];
    for component in &mut missing[..3] {
        let is_missing = input
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok();
        *component = is_missing && rgb_profile;
        if !is_missing {
            input.next()?;
        }
    }
    if input.is_exhausted() {
        return Ok(missing);
    }
    input.expect_delim('/')?;
    missing[3] = input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok();
    if !missing[3] {
        input.next()?;
    }
    if !input.is_exhausted() {
        return Err(input.new_custom_error(()));
    }
    Ok(missing)
}

fn parse_mix_components<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<[bool; 4], cssparser::ParseError<'i, ()>> {
    input.expect_ident_matching("in")?;
    let space = input.expect_ident_cloned()?.to_ascii_lowercase();
    if matches!(space.as_str(), "lch" | "oklch" | "hsl" | "hwb") {
        let _ = input.try_parse(|input| {
            input.expect_ident_cloned()?;
            input.expect_ident_matching("hue")
        });
    }
    input.expect_comma()?;
    let first = parse_mix_color(input, depth)?;
    input.expect_comma()?;
    let second = parse_mix_color(input, depth)?;
    if !input.is_exhausted() {
        return Err(input.new_custom_error(()));
    }
    let mut missing = [false; 4];
    if is_rgb_profile(&space) {
        for index in 0..3 {
            missing[index] = first[index] && second[index];
        }
    }
    missing[3] = first[3] && second[3];
    Ok(missing)
}

fn parse_mix_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<[bool; 4], cssparser::ParseError<'i, ()>> {
    let start = input.position();
    match input.next()? {
        Token::Function(_) | Token::ParenthesisBlock => {
            input.parse_nested_block(|nested| consume_all_tokens(nested, 0))?;
        }
        _ => {}
    }
    let end = input.position();
    let color = input.slice(start..end).trim();
    let _ = input.try_parse(|input| input.expect_percentage());
    Ok(parse_missing_components(color, depth))
}

fn is_rgb_profile(profile: &str) -> bool {
    matches!(
        profile,
        "srgb"
            | "srgb-linear"
            | "display-p3"
            | "display-p3-linear"
            | "a98-rgb"
            | "prophoto-rgb"
            | "rec2020"
    )
}

fn consume_all_tokens<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<(), cssparser::ParseError<'i, ()>> {
    if depth > 32 {
        return Err(input.new_custom_error(()));
    }
    while !input.is_exhausted() {
        match input.next()? {
            Token::Function(_) | Token::ParenthesisBlock => {
                input.parse_nested_block(|nested| consume_all_tokens(nested, depth + 1))?;
            }
            _ => {}
        }
    }
    Ok(())
}
