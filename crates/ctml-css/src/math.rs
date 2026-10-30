#[derive(Clone, Copy, Debug, PartialEq)]
enum MathValue {
    Number(f32),
    Dimension(f32),
}

impl MathValue {
    fn value(self) -> f32 {
        match self {
            Self::Number(value) | Self::Dimension(value) => value,
        }
    }

    fn map(self, map: impl FnOnce(f32) -> f32) -> Option<Self> {
        let value = map(self.value());
        value.is_finite().then_some(match self {
            Self::Number(_) => Self::Number(value),
            Self::Dimension(_) => Self::Dimension(value),
        })
    }
}

pub(crate) fn is_math_function(value: &str) -> bool {
    let value = value.trim_start();
    ["calc(", "min(", "max(", "clamp(", "round(", "mod(", "rem("]
        .iter()
        .any(|function| {
            value
                .get(..function.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(function))
        })
}

pub fn parse_math_expression(
    value: &str,
    parse_dimension: impl FnMut(&str) -> Option<f32>,
) -> Option<f32> {
    match parse_math_value(value, parse_dimension)? {
        MathValue::Dimension(value) if value.is_finite() => Some(value),
        _ => None,
    }
}

pub fn parse_math_number(value: &str) -> Option<f32> {
    match parse_math_value(value, |_| None)? {
        MathValue::Number(value) if value.is_finite() => Some(value),
        _ => None,
    }
}

fn parse_math_value(
    value: &str,
    parse_dimension: impl FnMut(&str) -> Option<f32>,
) -> Option<MathValue> {
    if !is_math_function(value) {
        return None;
    }
    let mut parser = MathParser {
        input: value,
        cursor: 0,
        parse_dimension,
    };
    let result = parser.parse_value()?;
    parser.skip_whitespace();
    (parser.cursor == parser.input.len()).then_some(result)
}

#[derive(Clone, Copy)]
enum RoundingStrategy {
    Nearest,
    Up,
    Down,
    ToZero,
}

struct MathParser<'a, F> {
    input: &'a str,
    cursor: usize,
    parse_dimension: F,
}

impl<F> MathParser<'_, F>
where
    F: FnMut(&str) -> Option<f32>,
{
    fn parse_value(&mut self) -> Option<MathValue> {
        self.skip_whitespace();
        let value = self.parse_sum()?;
        self.skip_whitespace();
        if self.cursor == self.input.len() || self.peek() == Some(b',') || self.peek() == Some(b')')
        {
            return Some(value);
        }
        None
    }

    fn parse_sum(&mut self) -> Option<MathValue> {
        let mut value = self.parse_product()?;
        loop {
            self.skip_whitespace();
            let Some(operator @ (b'+' | b'-')) = self.peek() else {
                return Some(value);
            };
            if !self.whitespace_around_operator() {
                return None;
            }
            self.cursor += 1;
            let right = self.parse_product()?;
            if std::mem::discriminant(&value) != std::mem::discriminant(&right) {
                return None;
            }
            let right = right.value();
            value = value.map(|left| {
                if operator == b'+' {
                    left + right
                } else {
                    left - right
                }
            })?;
        }
    }

    fn parse_product(&mut self) -> Option<MathValue> {
        let mut value = self.parse_unary()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some(b'*') => {
                    self.cursor += 1;
                    let right = self.parse_unary()?;
                    value = match (value, right) {
                        (MathValue::Number(left), MathValue::Number(right)) => {
                            MathValue::Number(left * right)
                        }
                        (MathValue::Dimension(left), MathValue::Number(right))
                        | (MathValue::Number(right), MathValue::Dimension(left)) => {
                            MathValue::Dimension(left * right)
                        }
                        _ => return None,
                    }
                    .map(|value| value)?;
                }
                Some(b'/') => {
                    self.cursor += 1;
                    let right = self.parse_unary()?;
                    let MathValue::Number(divisor) = right else {
                        return None;
                    };
                    if divisor == 0.0 {
                        return None;
                    }
                    value = value.map(|left| left / divisor)?;
                }
                _ => return Some(value),
            }
        }
    }

    fn parse_unary(&mut self) -> Option<MathValue> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'+') => {
                self.cursor += 1;
                self.parse_unary()
            }
            Some(b'-') => {
                self.cursor += 1;
                self.parse_unary()?.map(|value| -value)
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Option<MathValue> {
        self.skip_whitespace();
        match self.peek()? {
            b'(' => {
                self.cursor += 1;
                let value = self.parse_sum()?;
                self.skip_whitespace();
                self.consume(b')').then_some(value)
            }
            byte if byte.is_ascii_alphabetic() => self.parse_function(),
            _ => self.parse_numeric_token(),
        }
    }

    fn parse_function(&mut self) -> Option<MathValue> {
        let start = self.cursor;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            self.cursor += 1;
        }
        let function = self.input.get(start..self.cursor)?.to_ascii_lowercase();
        self.skip_whitespace();
        if !self.consume(b'(') {
            return None;
        }
        match function.as_str() {
            "calc" => {
                let value = self.parse_sum()?;
                self.skip_whitespace();
                self.consume(b')').then_some(value)
            }
            "min" | "max" | "clamp" => {
                let mut values = vec![self.parse_sum()?];
                loop {
                    self.skip_whitespace();
                    if self.consume(b')') {
                        break;
                    }
                    if !self.consume(b',') {
                        return None;
                    }
                    values.push(self.parse_sum()?);
                }
                self.evaluate_function(&function, &values)
            }
            "round" => self.parse_round_function(),
            "mod" => self.parse_remainder_function(true),
            "rem" => self.parse_remainder_function(false),
            _ => None,
        }
    }

    fn parse_remainder_function(&mut self, euclidean: bool) -> Option<MathValue> {
        let dividend = self.parse_sum()?;
        self.skip_whitespace();
        if !self.consume(b',') {
            return None;
        }
        let divisor = self.parse_sum()?;
        self.skip_whitespace();
        if !self.consume(b')')
            || std::mem::discriminant(&dividend) != std::mem::discriminant(&divisor)
            || divisor.value() == 0.0
        {
            return None;
        }

        let remainder = dividend.value() % divisor.value();
        let result = if euclidean
            && remainder != 0.0
            && remainder.is_sign_negative() != divisor.value().is_sign_negative()
        {
            remainder + divisor.value()
        } else {
            remainder
        };
        dividend.map(|_| result)
    }

    fn parse_round_function(&mut self) -> Option<MathValue> {
        self.skip_whitespace();
        let strategy_start = self.cursor;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'-')
        {
            self.cursor += 1;
        }
        let strategy_name = self
            .input
            .get(strategy_start..self.cursor)?
            .to_ascii_lowercase();
        self.skip_whitespace();
        let strategy = if self.consume(b',') {
            match strategy_name.as_str() {
                "nearest" => RoundingStrategy::Nearest,
                "up" => RoundingStrategy::Up,
                "down" => RoundingStrategy::Down,
                "to-zero" => RoundingStrategy::ToZero,
                _ => return None,
            }
        } else {
            self.cursor = strategy_start;
            RoundingStrategy::Nearest
        };

        let value = self.parse_sum()?;
        self.skip_whitespace();
        let interval = if self.consume(b',') {
            self.parse_sum()?
        } else {
            match value {
                MathValue::Number(_) => MathValue::Number(1.0),
                MathValue::Dimension(_) => return None,
            }
        };
        self.skip_whitespace();
        if !self.consume(b')')
            || std::mem::discriminant(&value) != std::mem::discriminant(&interval)
            || interval.value() == 0.0
        {
            return None;
        }

        let step = interval.value().abs();
        let quotient = value.value() / step;
        if !quotient.is_finite() {
            return None;
        }
        let lower = quotient.floor() * step;
        let upper = lower + step;
        let rounded = if lower == value.value() || upper == value.value() {
            value.value()
        } else {
            match strategy {
                RoundingStrategy::Nearest => {
                    if (value.value() - lower).abs() < (upper - value.value()).abs() {
                        lower
                    } else {
                        upper
                    }
                }
                RoundingStrategy::Up => upper,
                RoundingStrategy::Down => lower,
                RoundingStrategy::ToZero => {
                    if lower.abs() < upper.abs() {
                        lower
                    } else {
                        upper
                    }
                }
            }
        };
        value.map(|_| rounded)
    }

    fn evaluate_function(&self, function: &str, values: &[MathValue]) -> Option<MathValue> {
        let expected_length = match function {
            "clamp" => 3,
            _ => 1,
        };
        if values.len() < expected_length
            || (function == "clamp" && values.len() != expected_length)
            || values
                .iter()
                .any(|value| std::mem::discriminant(value) != std::mem::discriminant(&values[0]))
        {
            return None;
        }
        let selected = match function {
            "min" => values.iter().map(|value| value.value()).reduce(f32::min)?,
            "max" => values.iter().map(|value| value.value()).reduce(f32::max)?,
            "clamp" => values[0]
                .value()
                .max(values[1].value().min(values[2].value())),
            _ => return None,
        };
        values[0].map(|_| selected)
    }

    fn parse_numeric_token(&mut self) -> Option<MathValue> {
        let start = self.cursor;
        let bytes = self.input.as_bytes();
        let mut cursor = self.cursor;
        let mut digits = 0;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
            digits += 1;
        }
        if bytes.get(cursor) == Some(&b'.') {
            cursor += 1;
            while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
                cursor += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return None;
        }
        if bytes
            .get(cursor)
            .is_some_and(|byte| matches!(byte, b'e' | b'E'))
        {
            let exponent_start = cursor;
            cursor += 1;
            if bytes
                .get(cursor)
                .is_some_and(|byte| matches!(byte, b'+' | b'-'))
            {
                cursor += 1;
            }
            let digit_start = cursor;
            while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
                cursor += 1;
            }
            if cursor == digit_start {
                cursor = exponent_start;
            }
        }
        let number = self.input.get(start..cursor)?.parse::<f32>().ok()?;
        if !number.is_finite() {
            return None;
        }
        let suffix_start = cursor;
        while bytes
            .get(cursor)
            .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'%')
        {
            cursor += 1;
        }
        self.cursor = cursor;
        if suffix_start == cursor {
            Some(MathValue::Number(number))
        } else {
            let dimension = self.input.get(start..cursor)?;
            (self.parse_dimension)(dimension)
                .filter(|value| value.is_finite())
                .map(MathValue::Dimension)
        }
    }

    fn whitespace_around_operator(&self) -> bool {
        let before = self.input.as_bytes().get(self.cursor.wrapping_sub(1));
        let after = self.input.as_bytes().get(self.cursor + 1);
        before.is_some_and(u8::is_ascii_whitespace) && after.is_some_and(u8::is_ascii_whitespace)
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.cursor += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.cursor).copied()
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod tests;
