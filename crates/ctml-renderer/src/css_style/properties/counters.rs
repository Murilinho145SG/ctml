use cssparser::{Parser, ParserInput};

use crate::style::TextStyle;

#[derive(Clone, Copy)]
pub(super) enum CounterProperty {
    Reset,
    Increment,
    Set,
}

pub(super) fn apply_counter_property(
    style: &mut TextStyle,
    property: CounterProperty,
    value: &str,
) {
    let parsed = parse_counter_list(value, property);
    let Some(parsed) = parsed else {
        return;
    };
    match property {
        CounterProperty::Reset => style.counter_reset = parsed,
        CounterProperty::Increment => style.counter_increment = parsed,
        CounterProperty::Set => style.counter_set = parsed,
    }
}

fn parse_counter_list(value: &str, property: CounterProperty) -> Option<Vec<(String, i32)>> {
    let mut source = ParserInput::new(value);
    let mut parser = Parser::new(&mut source);
    parser.skip_whitespace();
    if parser
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        parser.skip_whitespace();
        return parser.is_exhausted().then(Vec::new);
    }

    let mut counters = Vec::new();
    while !parser.is_exhausted() {
        let name = parser.expect_ident_cloned().ok()?.to_string();
        let default = match property {
            CounterProperty::Reset | CounterProperty::Set => 0,
            CounterProperty::Increment => 1,
        };
        let value = parser.try_parse(Parser::expect_integer).unwrap_or(default);
        counters.push((name, value));
        parser.skip_whitespace();
    }
    (!counters.is_empty()).then_some(counters)
}
