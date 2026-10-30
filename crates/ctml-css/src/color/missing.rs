use cssparser::{Parser, ParserInput, Token};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ComponentSpace {
    Rgb,
    Xyz,
    Lab,
    Lch,
    Oklab,
    Oklch,
}

#[derive(Clone, Copy)]
pub(super) struct MissingComponents {
    space: Option<ComponentSpace>,
    channels: [bool; 3],
    pub(super) alpha: bool,
}

impl MissingComponents {
    fn empty() -> Self {
        Self {
            space: None,
            channels: [false; 3],
            alpha: false,
        }
    }

    pub(super) fn channels_for(self, space: Option<ComponentSpace>) -> [bool; 3] {
        if self.space == space {
            self.channels
        } else {
            [false; 3]
        }
    }
}

pub(super) fn parse_missing_components(value: &str) -> MissingComponents {
    parse_missing(value, 0)
}

fn parse_missing(value: &str, depth: usize) -> MissingComponents {
    if depth > 32 {
        return MissingComponents::empty();
    }
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    let Ok(function) = parser.expect_function() else {
        return MissingComponents::empty();
    };
    let function = function.to_ascii_lowercase();
    let missing = match function.as_str() {
        "color" => parser
            .parse_nested_block(parse_color_components)
            .unwrap_or_else(|_| MissingComponents::empty()),
        "lab" | "lch" | "oklab" | "oklch" => parser
            .parse_nested_block(|input| parse_lab_components(input, &function))
            .unwrap_or_else(|_| MissingComponents::empty()),
        "color-mix" => parser
            .parse_nested_block(|input| parse_mix_components(input, depth + 1))
            .unwrap_or_else(|_| MissingComponents::empty()),
        _ => MissingComponents::empty(),
    };
    if parser.is_exhausted() {
        missing
    } else {
        MissingComponents::empty()
    }
}

fn parse_color_components<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<MissingComponents, cssparser::ParseError<'i, ()>> {
    let profile = input.expect_ident_cloned()?.to_ascii_lowercase();
    let space = component_space(&profile);
    let mut missing = MissingComponents {
        space,
        ..MissingComponents::empty()
    };
    for component in &mut missing.channels {
        let is_missing = input
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok();
        *component = is_missing && space.is_some();
        if !is_missing {
            input.next()?;
        }
    }
    parse_alpha_component(input, &mut missing)?;
    Ok(missing)
}

fn parse_lab_components<'i, 't>(
    input: &mut Parser<'i, 't>,
    function: &str,
) -> Result<MissingComponents, cssparser::ParseError<'i, ()>> {
    let space = component_space(function);
    let mut missing = MissingComponents {
        space,
        ..MissingComponents::empty()
    };
    for component in &mut missing.channels {
        let is_missing = input
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok();
        *component = is_missing;
        if !is_missing {
            input.next()?;
        }
    }
    parse_alpha_component(input, &mut missing)?;
    Ok(missing)
}

fn parse_alpha_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    missing: &mut MissingComponents,
) -> Result<(), cssparser::ParseError<'i, ()>> {
    if input.is_exhausted() {
        return Ok(());
    }
    input.expect_delim('/')?;
    missing.alpha = input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok();
    if !missing.alpha {
        input.next()?;
    }
    if !input.is_exhausted() {
        return Err(input.new_custom_error(()));
    }
    Ok(())
}

fn parse_mix_components<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<MissingComponents, cssparser::ParseError<'i, ()>> {
    input.expect_ident_matching("in")?;
    let interpolation_space = input.expect_ident_cloned()?.to_ascii_lowercase();
    let space = component_space(&interpolation_space);
    if matches!(
        interpolation_space.as_str(),
        "lch" | "oklch" | "hsl" | "hwb"
    ) {
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
    let first_channels = first.channels_for(space);
    let second_channels = second.channels_for(space);
    Ok(MissingComponents {
        space,
        channels: std::array::from_fn(|index| first_channels[index] && second_channels[index]),
        alpha: first.alpha && second.alpha,
    })
}

fn parse_mix_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: usize,
) -> Result<MissingComponents, cssparser::ParseError<'i, ()>> {
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
    Ok(parse_missing(color, depth))
}

pub(super) fn component_space(space: &str) -> Option<ComponentSpace> {
    match space {
        "srgb" | "srgb-linear" | "display-p3" | "display-p3-linear" | "a98-rgb"
        | "prophoto-rgb" | "rec2020" | "rec2100-pq" | "rec2100-hlg" | "rec2100-linear" => {
            Some(ComponentSpace::Rgb)
        }
        "xyz" | "xyz-d50" | "xyz-d65" => Some(ComponentSpace::Xyz),
        "lab" => Some(ComponentSpace::Lab),
        "lch" => Some(ComponentSpace::Lch),
        "oklab" => Some(ComponentSpace::Oklab),
        "oklch" => Some(ComponentSpace::Oklch),
        _ => None,
    }
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
