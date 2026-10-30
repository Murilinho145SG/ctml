use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn manual_soft_hyphen_draws_a_hyphen_only_when_the_line_breaks_there() {
    let soft_hyphen = [paragraph(vec![UiNode::Text(
        "hy\u{00ad}phenation".to_owned(),
    )])];
    let explicit_break = [paragraph(vec![
        UiNode::Text("hy".to_owned()),
        UiNode::Element {
            tag: "br".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        },
        UiNode::Text("phenation".to_owned()),
    ])];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "width".to_owned(),
            value: "35px".to_owned(),
        }],
    }];
    let mut soft_page = PageRasterizer::new(&soft_hyphen, &stylesheet);
    let mut explicit_page = PageRasterizer::new(&explicit_break, &stylesheet);

    let soft_pixels = soft_page.snapshot_pixels(140, 100);
    let explicit_pixels = explicit_page.snapshot_pixels(140, 100);

    assert_ne!(soft_pixels, explicit_pixels);
}

#[test]
fn manual_soft_hyphen_is_invisible_when_the_word_stays_on_one_line() {
    let soft_hyphen = [paragraph(vec![UiNode::Text(
        "hy\u{00ad}phenation".to_owned(),
    )])];
    let plain_text = [paragraph(vec![UiNode::Text("hyphenation".to_owned())])];
    let mut soft_page = PageRasterizer::new(&soft_hyphen, &[]);
    let mut plain_page = PageRasterizer::new(&plain_text, &[]);

    assert_eq!(
        soft_page.snapshot_pixels(200, 80),
        plain_page.snapshot_pixels(200, 80)
    );
}

#[test]
fn hyphens_none_suppresses_manual_soft_hyphen_breaks() {
    let soft_hyphen = [paragraph(vec![UiNode::Text(
        "hy\u{00ad}phenation".to_owned(),
    )])];
    let plain_text = [paragraph(vec![UiNode::Text("hyphenation".to_owned())])];
    let stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![
            StyleDeclaration {
                property: "width".to_owned(),
                value: "35px".to_owned(),
            },
            StyleDeclaration {
                property: "hyphens".to_owned(),
                value: "none".to_owned(),
            },
        ],
    }];
    let mut soft_page = PageRasterizer::new(&soft_hyphen, &stylesheet);
    let mut plain_page = PageRasterizer::new(&plain_text, &stylesheet);

    assert_eq!(
        soft_page.snapshot_pixels(140, 100),
        plain_page.snapshot_pixels(140, 100)
    );
}

fn paragraph(children: Vec<UiNode>) -> UiNode {
    UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children,
    }
}
