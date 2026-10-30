pub(super) fn format_marker(
    list_style_type: Option<&str>,
    ordered_parent: bool,
    index: usize,
) -> Option<String> {
    let list_style_type =
        list_style_type.unwrap_or(if ordered_parent { "decimal" } else { "disc" });
    if let Some(marker) = list_style_type.strip_prefix("custom:") {
        return Some(format!("{marker} "));
    }
    Some(match list_style_type {
        "none" => return None,
        "disc" => "• ".to_owned(),
        "circle" => "◦ ".to_owned(),
        "square" => "▪ ".to_owned(),
        "disclosure-open" => "▾ ".to_owned(),
        "disclosure-closed" => "▸ ".to_owned(),
        "decimal" => format!("{index}. "),
        "decimal-leading-zero" => format!("{index:02}. "),
        "lower-alpha" => format!("{}. ", alphabetic_marker(index, false)),
        "upper-alpha" => format!("{}. ", alphabetic_marker(index, true)),
        "lower-greek" => format!(
            "{}. ",
            alphabetic_marker_from(
                index,
                &[
                    'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π',
                    'ρ', 'σ', 'τ', 'υ', 'φ', 'χ', 'ψ', 'ω',
                ]
            )
        ),
        "lower-roman" => format!("{}. ", roman_marker(index).to_ascii_lowercase()),
        "upper-roman" => format!("{}. ", roman_marker(index)),
        "armenian" | "upper-armenian" | "lower-armenian" | "hebrew" | "georgian"
        | "cjk-decimal" | "cjk-ideographic" | "cjk-earthly-branch" | "cjk-heavenly-stem"
        | "lower-latin" | "upper-latin" | "hiragana" | "hiragana-iroha" | "katakana"
        | "katakana-iroha" => {
            let marker = format_counter_value(list_style_type, index as i32)
                .unwrap_or_else(|| index.to_string());
            let suffix = if matches!(
                list_style_type,
                "cjk-decimal"
                    | "cjk-ideographic"
                    | "cjk-earthly-branch"
                    | "cjk-heavenly-stem"
                    | "hiragana"
                    | "hiragana-iroha"
                    | "katakana"
                    | "katakana-iroha"
            ) {
                "、 "
            } else {
                ". "
            };
            format!("{marker}{suffix}")
        }
        _ => return None,
    })
}

pub(super) fn format_counter_value(style: &str, value: i32) -> Option<String> {
    let positive = usize::try_from(value).ok().filter(|value| *value > 0);
    match style {
        "decimal" => Some(value.to_string()),
        "disclosure-open" => Some("▾".to_owned()),
        "disclosure-closed" => Some("▸".to_owned()),
        "decimal-leading-zero" if (0..10).contains(&value) => Some(format!("0{value}")),
        "decimal-leading-zero" => Some(value.to_string()),
        "lower-alpha" => {
            Some(positive.map_or_else(|| value.to_string(), |n| alphabetic_marker(n, false)))
        }
        "lower-latin" => {
            Some(positive.map_or_else(|| value.to_string(), |n| alphabetic_marker(n, false)))
        }
        "upper-alpha" => {
            Some(positive.map_or_else(|| value.to_string(), |n| alphabetic_marker(n, true)))
        }
        "upper-latin" => {
            Some(positive.map_or_else(|| value.to_string(), |n| alphabetic_marker(n, true)))
        }
        "lower-greek" => Some(positive.map_or_else(
            || value.to_string(),
            |n| {
                alphabetic_marker_from(
                    n,
                    &[
                        'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο',
                        'π', 'ρ', 'σ', 'τ', 'υ', 'φ', 'χ', 'ψ', 'ω',
                    ],
                )
            },
        )),
        "lower-roman" | "upper-roman" => Some(positive.map_or_else(
            || value.to_string(),
            |n| {
                let roman = roman_marker(n);
                if style == "lower-roman" {
                    roman.to_ascii_lowercase()
                } else {
                    roman
                }
            },
        )),
        "armenian" | "upper-armenian" => {
            additive_numeral(value, &ARMENIAN_SYMBOLS, 9999).or_else(|| Some(value.to_string()))
        }
        "lower-armenian" => additive_numeral(value, &LOWER_ARMENIAN_SYMBOLS, 9999)
            .or_else(|| Some(value.to_string())),
        "hebrew" => {
            additive_numeral(value, &HEBREW_SYMBOLS, 10_999).or_else(|| Some(value.to_string()))
        }
        "georgian" => {
            additive_numeral(value, &GEORGIAN_SYMBOLS, 19_999).or_else(|| Some(value.to_string()))
        }
        "cjk-decimal" | "cjk-ideographic" => Some(cjk_decimal(value)),
        "cjk-earthly-branch" => Some(fixed_cjk(value, "子丑寅卯辰巳午未申酉戌亥")),
        "cjk-heavenly-stem" => Some(fixed_cjk(value, "甲乙丙丁戊己庚辛壬癸")),
        "hiragana" => Some(alphabetic_string(
            value,
            "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわゐゑをん",
        )),
        "hiragana-iroha" => Some(alphabetic_string(
            value,
            "いろはにほへとちりぬるをわかよたれそつねならむうゐのおくやまけふこえてあさきゆめみしゑひもせす",
        )),
        "katakana" => Some(alphabetic_string(
            value,
            "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヰヱヲン",
        )),
        "katakana-iroha" => Some(alphabetic_string(
            value,
            "イロハニホヘトチリヌルヲワカヨタレソツネナラムウヰノオクヤマケフコエテアサキユメミシヱヒモセス",
        )),
        "arabic-indic" => Some(localized_decimal(value, '٠')),
        "bengali" => Some(localized_decimal(value, '০')),
        "cambodian" | "khmer" => Some(localized_decimal(value, '០')),
        "devanagari" => Some(localized_decimal(value, '०')),
        "gujarati" => Some(localized_decimal(value, '૦')),
        "gurmukhi" => Some(localized_decimal(value, '੦')),
        "kannada" => Some(localized_decimal(value, '೦')),
        "lao" => Some(localized_decimal(value, '໐')),
        "malayalam" => Some(localized_decimal(value, '൦')),
        "mongolian" => Some(localized_decimal(value, '᠐')),
        "myanmar" => Some(localized_decimal(value, '၀')),
        "oriya" => Some(localized_decimal(value, '୦')),
        "persian" => Some(localized_decimal(value, '۰')),
        "tamil" => Some(localized_decimal(value, '௦')),
        "telugu" => Some(localized_decimal(value, '౦')),
        "thai" => Some(localized_decimal(value, '๐')),
        "tibetan" => Some(localized_decimal(value, '༠')),
        "ethiopic-numeric" => Some(ethiopic_numeric(value)),
        _ => Some(value.to_string()),
    }
}

fn ethiopic_numeric(value: i32) -> String {
    if value <= 0 {
        return cjk_decimal(value);
    }
    if value == 1 {
        return "፩".to_owned();
    }

    let mut groups = Vec::new();
    let mut remaining = value as u32;
    while remaining > 0 {
        groups.push(remaining % 100);
        remaining /= 100;
    }

    let digits = ['፩', '፪', '፫', '፬', '፭', '፮', '፯', '፰', '፱'];
    let tens = ['፲', '፳', '፴', '፵', '፶', '፷', '፸', '፹', '፺'];
    let mut result = String::new();
    for (index, &group) in groups.iter().enumerate().rev() {
        let most_significant = index == groups.len() - 1;
        let suppress_digits = group == 0 || (group == 1 && (most_significant || index % 2 == 1));
        if !suppress_digits {
            let tens_digit = (group / 10) as usize;
            let units_digit = (group % 10) as usize;
            if tens_digit > 0 {
                result.push(tens[tens_digit - 1]);
            }
            if units_digit > 0 {
                result.push(digits[units_digit - 1]);
            }
        }
        if index % 2 == 1 && group != 0 {
            result.push('፻');
        } else if index % 2 == 0 && index != 0 {
            result.push('፼');
        }
    }
    result
}

const ARMENIAN_SYMBOLS: &[(i32, &str)] = &[
    (9000, "Ք"),
    (8000, "Փ"),
    (7000, "Ւ"),
    (6000, "Ց"),
    (5000, "Ր"),
    (4000, "Տ"),
    (3000, "Վ"),
    (2000, "Ս"),
    (1000, "Ռ"),
    (900, "Ջ"),
    (800, "Պ"),
    (700, "Չ"),
    (600, "Ո"),
    (500, "Շ"),
    (400, "Ն"),
    (300, "Յ"),
    (200, "Մ"),
    (100, "Ճ"),
    (90, "Ղ"),
    (80, "Ձ"),
    (70, "Հ"),
    (60, "Կ"),
    (50, "Ծ"),
    (40, "Խ"),
    (30, "Լ"),
    (20, "Ի"),
    (10, "Ժ"),
    (9, "Թ"),
    (8, "Ը"),
    (7, "Է"),
    (6, "Զ"),
    (5, "Ե"),
    (4, "Դ"),
    (3, "Գ"),
    (2, "Բ"),
    (1, "Ա"),
];

const LOWER_ARMENIAN_SYMBOLS: &[(i32, &str)] = &[
    (9000, "ք"),
    (8000, "փ"),
    (7000, "ւ"),
    (6000, "ց"),
    (5000, "ր"),
    (4000, "տ"),
    (3000, "վ"),
    (2000, "ս"),
    (1000, "ռ"),
    (900, "ջ"),
    (800, "պ"),
    (700, "չ"),
    (600, "ո"),
    (500, "շ"),
    (400, "ն"),
    (300, "յ"),
    (200, "մ"),
    (100, "ճ"),
    (90, "ղ"),
    (80, "ձ"),
    (70, "հ"),
    (60, "կ"),
    (50, "ծ"),
    (40, "խ"),
    (30, "լ"),
    (20, "ի"),
    (10, "ժ"),
    (9, "թ"),
    (8, "ը"),
    (7, "է"),
    (6, "զ"),
    (5, "ե"),
    (4, "դ"),
    (3, "գ"),
    (2, "բ"),
    (1, "ա"),
];

const HEBREW_SYMBOLS: &[(i32, &str)] = &[
    (10000, "י׳"),
    (9000, "ט׳"),
    (8000, "ח׳"),
    (7000, "ז׳"),
    (6000, "ו׳"),
    (5000, "ה׳"),
    (4000, "ד׳"),
    (3000, "ג׳"),
    (2000, "ב׳"),
    (1000, "א׳"),
    (400, "ת"),
    (300, "ש"),
    (200, "ר"),
    (100, "ק"),
    (90, "צ"),
    (80, "פ"),
    (70, "ע"),
    (60, "ס"),
    (50, "נ"),
    (40, "מ"),
    (30, "ל"),
    (20, "כ"),
    (19, "יט"),
    (18, "יח"),
    (17, "יז"),
    (16, "טז"),
    (15, "טו"),
    (10, "י"),
    (9, "ט"),
    (8, "ח"),
    (7, "ז"),
    (6, "ו"),
    (5, "ה"),
    (4, "ד"),
    (3, "ג"),
    (2, "ב"),
    (1, "א"),
];

const GEORGIAN_SYMBOLS: &[(i32, &str)] = &[
    (10_000, "\u{10f5}"),
    (9_000, "\u{10f0}"),
    (8_000, "\u{10ef}"),
    (7_000, "\u{10f4}"),
    (6_000, "\u{10ee}"),
    (5_000, "\u{10ed}"),
    (4_000, "\u{10ec}"),
    (3_000, "\u{10eb}"),
    (2_000, "\u{10ea}"),
    (1_000, "\u{10e9}"),
    (900, "\u{10e8}"),
    (800, "\u{10e7}"),
    (700, "\u{10e6}"),
    (600, "\u{10e5}"),
    (500, "\u{10e4}"),
    (400, "\u{10f3}"),
    (300, "\u{10e2}"),
    (200, "\u{10e1}"),
    (100, "\u{10e0}"),
    (90, "\u{10df}"),
    (80, "\u{10de}"),
    (70, "\u{10dd}"),
    (60, "\u{10f2}"),
    (50, "\u{10dc}"),
    (40, "\u{10db}"),
    (30, "\u{10da}"),
    (20, "\u{10d9}"),
    (10, "\u{10d8}"),
    (9, "\u{10d7}"),
    (8, "\u{10f1}"),
    (7, "\u{10d6}"),
    (6, "\u{10d5}"),
    (5, "\u{10d4}"),
    (4, "\u{10d3}"),
    (3, "\u{10d2}"),
    (2, "\u{10d1}"),
    (1, "\u{10d0}"),
];

fn additive_numeral(value: i32, symbols: &[(i32, &str)], max: i32) -> Option<String> {
    if !(1..=max).contains(&value) {
        return None;
    }
    let mut remaining = value;
    let mut result = String::new();
    for &(weight, symbol) in symbols {
        let count = remaining / weight;
        for _ in 0..count {
            result.push_str(symbol);
        }
        remaining %= weight;
        if remaining == 0 {
            return Some(result);
        }
    }
    None
}

fn cjk_decimal(value: i32) -> String {
    if value < 0 {
        return value.to_string();
    }
    let digits = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];
    value
        .unsigned_abs()
        .to_string()
        .bytes()
        .map(|digit| digits[usize::from(digit - b'0')])
        .collect()
}

fn localized_decimal(value: i32, zero: char) -> String {
    value
        .to_string()
        .chars()
        .map(|character| {
            character.to_digit(10).map_or(character, |digit| {
                char::from_u32(zero as u32 + digit).unwrap_or(character)
            })
        })
        .collect()
}

fn fixed_cjk(value: i32, symbols: &str) -> String {
    let symbols = symbols.chars().collect::<Vec<_>>();
    usize::try_from(value)
        .ok()
        .and_then(|value| value.checked_sub(1))
        .and_then(|index| symbols.get(index).copied())
        .map_or_else(|| cjk_decimal(value), |symbol| symbol.to_string())
}

fn alphabetic_string(value: i32, symbols: &str) -> String {
    let alphabet = symbols.chars().collect::<Vec<_>>();
    usize::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .map_or_else(
            || value.to_string(),
            |value| alphabetic_marker_from(value, &alphabet),
        )
}

fn alphabetic_marker(index: usize, uppercase: bool) -> String {
    let alphabet = if uppercase {
        ('A'..='Z').collect::<Vec<_>>()
    } else {
        ('a'..='z').collect::<Vec<_>>()
    };
    alphabetic_marker_from(index, &alphabet)
}

fn alphabetic_marker_from(mut index: usize, alphabet: &[char]) -> String {
    let mut marker = Vec::new();
    while index > 0 {
        let digit = (index - 1) % alphabet.len();
        marker.push(alphabet[digit]);
        index = (index - 1) / alphabet.len();
    }
    marker.iter().rev().collect()
}

fn roman_marker(index: usize) -> String {
    if !(1..=3999).contains(&index) {
        return index.to_string();
    }
    let mut value = index;
    let mut marker = String::new();
    for (amount, symbol) in [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while value >= amount {
            marker.push_str(symbol);
            value -= amount;
        }
    }
    marker
}
