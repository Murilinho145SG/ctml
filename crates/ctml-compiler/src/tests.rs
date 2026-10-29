use crate::{AttributeValue, parse};

#[test]
fn exposes_linked_stylesheets_in_header_order() {
    let document = parse(
        "<headers><stylesheet href=\"reset.css\"/><application/><stylesheet href=\"theme.css\"/></headers><body/>",
    )
    .expect("document should parse");

    let paths = document
        .stylesheets()
        .map(|stylesheet| match stylesheet.attribute("href") {
            Some(AttributeValue::String(path)) => path.as_str(),
            _ => panic!("stylesheet should have a string href"),
        })
        .collect::<Vec<_>>();

    assert_eq!(paths, ["reset.css", "theme.css"]);
    assert_eq!(
        document
            .stylesheet()
            .and_then(|stylesheet| stylesheet.attribute("href"))
            .and_then(AttributeValue::as_str),
        Some("reset.css")
    );
}
