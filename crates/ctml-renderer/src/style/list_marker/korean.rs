const HANGUL_FORMAL_SYMBOLS: &[(i32, &str)] = &[
    (9000, "구천"),
    (8000, "팔천"),
    (7000, "칠천"),
    (6000, "육천"),
    (5000, "오천"),
    (4000, "사천"),
    (3000, "삼천"),
    (2000, "이천"),
    (1000, "일천"),
    (900, "구백"),
    (800, "팔백"),
    (700, "칠백"),
    (600, "육백"),
    (500, "오백"),
    (400, "사백"),
    (300, "삼백"),
    (200, "이백"),
    (100, "일백"),
    (90, "구십"),
    (80, "팔십"),
    (70, "칠십"),
    (60, "육십"),
    (50, "오십"),
    (40, "사십"),
    (30, "삼십"),
    (20, "이십"),
    (10, "일십"),
    (9, "구"),
    (8, "팔"),
    (7, "칠"),
    (6, "육"),
    (5, "오"),
    (4, "사"),
    (3, "삼"),
    (2, "이"),
    (1, "일"),
    (0, "영"),
];

const HANJA_INFORMAL_SYMBOLS: &[(i32, &str)] = &[
    (9000, "九千"),
    (8000, "八千"),
    (7000, "七千"),
    (6000, "六千"),
    (5000, "五千"),
    (4000, "四千"),
    (3000, "三千"),
    (2000, "二千"),
    (1000, "千"),
    (900, "九百"),
    (800, "八百"),
    (700, "七百"),
    (600, "六百"),
    (500, "五百"),
    (400, "四百"),
    (300, "三百"),
    (200, "二百"),
    (100, "百"),
    (90, "九十"),
    (80, "八十"),
    (70, "七十"),
    (60, "六十"),
    (50, "五十"),
    (40, "四十"),
    (30, "三十"),
    (20, "二十"),
    (10, "十"),
    (9, "九"),
    (8, "八"),
    (7, "七"),
    (6, "六"),
    (5, "五"),
    (4, "四"),
    (3, "三"),
    (2, "二"),
    (1, "一"),
    (0, "零"),
];

const HANJA_FORMAL_SYMBOLS: &[(i32, &str)] = &[
    (9000, "九仟"),
    (8000, "八仟"),
    (7000, "七仟"),
    (6000, "六仟"),
    (5000, "五仟"),
    (4000, "四仟"),
    (3000, "參仟"),
    (2000, "貳仟"),
    (1000, "壹仟"),
    (900, "九百"),
    (800, "八百"),
    (700, "七百"),
    (600, "六百"),
    (500, "五百"),
    (400, "四百"),
    (300, "參百"),
    (200, "貳百"),
    (100, "壹百"),
    (90, "九拾"),
    (80, "八拾"),
    (70, "七拾"),
    (60, "六拾"),
    (50, "五拾"),
    (40, "四拾"),
    (30, "參拾"),
    (20, "貳拾"),
    (10, "壹拾"),
    (9, "九"),
    (8, "八"),
    (7, "七"),
    (6, "六"),
    (5, "五"),
    (4, "四"),
    (3, "參"),
    (2, "貳"),
    (1, "壹"),
    (0, "零"),
];

pub(super) fn format(style: &str, value: i32) -> String {
    if !(-9999..=9999).contains(&value) {
        return super::cjk_decimal(value);
    }
    let symbols = match style {
        "korean-hangul-formal" => HANGUL_FORMAL_SYMBOLS,
        "korean-hanja-formal" => HANJA_FORMAL_SYMBOLS,
        _ => HANJA_INFORMAL_SYMBOLS,
    };
    let negative = value < 0;
    let representation = additive_numeral(value.unsigned_abs() as i32, symbols).unwrap_or_default();
    if negative {
        format!("마이너스 {representation}")
    } else {
        representation
    }
}

fn additive_numeral(mut value: i32, symbols: &[(i32, &str)]) -> Option<String> {
    if value == 0 {
        return symbols
            .iter()
            .find_map(|(weight, symbol)| (*weight == 0).then(|| (*symbol).to_owned()));
    }
    let mut result = String::new();
    for &(weight, symbol) in symbols {
        if weight == 0 {
            continue;
        }
        let count = value / weight;
        for _ in 0..count {
            result.push_str(symbol);
        }
        value %= weight;
        if value == 0 {
            return Some(result);
        }
    }
    None
}
