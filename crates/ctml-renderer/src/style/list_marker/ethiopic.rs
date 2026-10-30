pub(super) fn format(value: i32) -> String {
    if value <= 0 {
        return super::cjk_decimal(value);
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
