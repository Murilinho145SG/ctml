use super::{MathParser, MathValue, RoundingStrategy};

impl<F> MathParser<'_, F>
where
    F: FnMut(&str) -> Option<f32>,
{
    pub(super) fn parse_sign_function(&mut self, absolute: bool) -> Option<MathValue> {
        let value = self.parse_sum()?;
        self.skip_whitespace();
        if !self.consume(b')') {
            return None;
        }
        if absolute {
            value.map(f32::abs)
        } else {
            let number = if value.value() == 0.0 {
                if value.value().is_sign_negative() {
                    -0.0
                } else {
                    0.0
                }
            } else if value.value().is_sign_negative() {
                -1.0
            } else {
                1.0
            };
            Some(MathValue::Number(number))
        }
    }

    pub(super) fn parse_trigonometric_function(&mut self, function: &str) -> Option<MathValue> {
        let first = self.parse_sum()?;
        self.skip_whitespace();
        if function == "atan2" {
            if !self.consume(b',') {
                return None;
            }
            let second = self.parse_sum()?;
            self.skip_whitespace();
            if !self.consume(b')')
                || std::mem::discriminant(&first) != std::mem::discriminant(&second)
                || !matches!(first, MathValue::Number(_) | MathValue::Dimension(_))
            {
                return None;
            }
            let radians = first.value().atan2(second.value());
            return radians.is_finite().then_some(MathValue::Angle(radians));
        }

        if !self.consume(b')') {
            return None;
        }
        match function {
            "sin" | "cos" | "tan" => {
                let radians = match first {
                    MathValue::Number(value) | MathValue::Angle(value) => value,
                    MathValue::Dimension(_) => return None,
                };
                let result = match function {
                    "sin" => radians.sin(),
                    "cos" => radians.cos(),
                    _ => radians.tan(),
                };
                result.is_finite().then_some(MathValue::Number(result))
            }
            "asin" | "acos" | "atan" => {
                let MathValue::Number(value) = first else {
                    return None;
                };
                let radians = match function {
                    "asin" => value.asin(),
                    "acos" => value.acos(),
                    _ => value.atan(),
                };
                radians.is_finite().then_some(MathValue::Angle(radians))
            }
            _ => None,
        }
    }

    pub(super) fn parse_remainder_function(&mut self, euclidean: bool) -> Option<MathValue> {
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

    pub(super) fn parse_round_function(&mut self) -> Option<MathValue> {
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
                MathValue::Dimension(_) | MathValue::Angle(_) => return None,
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
}
