const INFORMAL_SYMBOLS: &[(i32, &str)] = &[
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
    (0, "〇"),
];

const FORMAL_SYMBOLS: &[(i32, &str)] = &[
    (9000, "九阡"),
    (8000, "八阡"),
    (7000, "七阡"),
    (6000, "六阡"),
    (5000, "伍阡"),
    (4000, "四阡"),
    (3000, "参阡"),
    (2000, "弐阡"),
    (1000, "壱阡"),
    (900, "九百"),
    (800, "八百"),
    (700, "七百"),
    (600, "六百"),
    (500, "伍百"),
    (400, "四百"),
    (300, "参百"),
    (200, "弐百"),
    (100, "壱百"),
    (90, "九拾"),
    (80, "八拾"),
    (70, "七拾"),
    (60, "六拾"),
    (50, "伍拾"),
    (40, "四拾"),
    (30, "参拾"),
    (20, "弐拾"),
    (10, "壱拾"),
    (9, "九"),
    (8, "八"),
    (7, "七"),
    (6, "六"),
    (5, "伍"),
    (4, "四"),
    (3, "参"),
    (2, "弐"),
    (1, "壱"),
    (0, "零"),
];

pub(super) fn format(style: &str, value: i32) -> String {
    if !(-9999..=9999).contains(&value) {
        return super::cjk_decimal(value);
    }
    let negative = value < 0;
    let value = value.unsigned_abs() as i32;
    let symbols = if style == "japanese-formal" {
        FORMAL_SYMBOLS
    } else {
        INFORMAL_SYMBOLS
    };
    let representation = additive_numeral(value, symbols).unwrap_or_default();
    if negative {
        format!("マイナス{representation}")
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
