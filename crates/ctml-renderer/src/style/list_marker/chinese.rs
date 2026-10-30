const DIGITS: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
const FORMAL_DIGITS: [&str; 10] = ["零", "壹", "贰", "叁", "肆", "伍", "陆", "柒", "捌", "玖"];
const TRADITIONAL_FORMAL_DIGITS: [&str; 10] =
    ["零", "壹", "貳", "參", "肆", "伍", "陸", "柒", "捌", "玖"];

pub(super) fn format(style: &str, value: i32) -> String {
    if !(-9999..=9999).contains(&value) {
        return super::cjk_decimal(value);
    }

    let simplified = style.starts_with("simp-");
    let formal = style.contains("-formal");
    let digits = match (simplified, formal) {
        (true, true) => &FORMAL_DIGITS,
        (false, true) => &TRADITIONAL_FORMAL_DIGITS,
        _ => &DIGITS,
    };
    let marker = match (simplified, formal) {
        (_, true) => ["", "拾", "佰", "仟"],
        _ => ["", "十", "百", "千"],
    };

    if value == 0 {
        return "零".to_owned();
    }

    let value_string = value.unsigned_abs().to_string();
    let source_digits = value_string.bytes().map(|digit| digit - b'0');
    let mut output = String::new();
    let mut pending_zero = false;
    for (index, digit) in source_digits.enumerate() {
        let place = value_string.len() - index - 1;
        if digit == 0 {
            pending_zero = !output.is_empty();
            continue;
        }
        if pending_zero {
            output.push_str("零");
            pending_zero = false;
        }
        if !(place == 1 && digit == 1 && !formal && (10..20).contains(&value.unsigned_abs())) {
            output.push_str(digits[usize::from(digit)]);
        }
        output.push_str(marker[place]);
    }

    if value < 0 {
        let negative = if simplified { "负" } else { "負" };
        format!("{negative}{output}")
    } else {
        output
    }
}
