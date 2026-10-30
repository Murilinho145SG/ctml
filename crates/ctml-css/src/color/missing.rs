use cssparser::{Parser, ParserInput};

pub(super) fn parse_missing_rgb_components(value: &str) -> [bool; 4] {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    if !parser
        .expect_function()
        .ok()
        .is_some_and(|name| name.eq_ignore_ascii_case("color"))
    {
        return [false; 4];
    }
    parser
        .parse_nested_block(|input| -> Result<_, cssparser::ParseError<'_, ()>> {
            let profile = input.expect_ident_cloned()?.to_ascii_lowercase();
            let rgb_profile = matches!(
                profile.as_str(),
                "srgb"
                    | "srgb-linear"
                    | "display-p3"
                    | "display-p3-linear"
                    | "a98-rgb"
                    | "prophoto-rgb"
                    | "rec2020"
            );
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
        })
        .unwrap_or([false; 4])
}
