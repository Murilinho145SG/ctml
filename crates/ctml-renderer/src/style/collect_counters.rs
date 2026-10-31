use std::collections::HashMap;

pub(super) fn set_counter(
    counters: &mut HashMap<String, Vec<i32>>,
    scope: &mut Vec<String>,
    name: &str,
    value: i32,
) {
    let values = counters.entry(name.to_owned()).or_insert_with(|| {
        scope.push(name.to_owned());
        vec![value]
    });
    if let Some(current) = values.last_mut() {
        *current = value;
    }
}
