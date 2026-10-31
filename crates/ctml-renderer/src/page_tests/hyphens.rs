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
    let ink_pixels = |pixels: &[u8]| {
        pixels
            .chunks_exact(4)
            .filter(|pixel| pixel[..3] != [255, 255, 255])
            .count()
    };
    assert!(
        soft_pixels != explicit_pixels,
        "soft-hyphen and explicit-break renders should differ; ink pixels: soft={}, explicit={}",
        ink_pixels(&soft_pixels),
        ink_pixels(&explicit_pixels)
    );
}

#[test]
fn br_starts_a_new_line() {
    let nodes = [paragraph(vec![
        UiNode::Text("before".to_owned()),
        UiNode::Element {
            tag: "br".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        },
        UiNode::Text("after".to_owned()),
    ])];
    let mut page = PageRasterizer::new(&nodes, &[]);

    page.snapshot_pixels(200, 80);

    let lines = page
        .buffer
        .layout_runs()
        .map(|line| line.text.to_owned())
        .collect::<Vec<_>>();
    assert_eq!(lines, ["before", "after"]);
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

#[test]
fn automatic_hyphenation_uses_the_declared_language_dictionary() {
    let nodes = [paragraph_with_language(
        "en-US",
        vec![UiNode::Text("hyphenation".to_owned())],
    )];
    let manual = [paragraph_with_language(
        "en-US",
        vec![UiNode::Text("hy\u{00ad}phen\u{00ad}ation".to_owned())],
    )];
    let automatic_stylesheet = [
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "width".to_owned(),
                value: "35px".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "hyphens".to_owned(),
                value: "auto".to_owned(),
            }],
        },
    ];
    let manual_stylesheet = [StyleRule {
        selectors: vec!["p".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "width".to_owned(),
            value: "35px".to_owned(),
        }],
    }];
    let mut automatic_page = PageRasterizer::new(&nodes, &automatic_stylesheet);
    let mut manual_page = PageRasterizer::new(&manual, &manual_stylesheet);

    assert!(
        automatic_page
            .runs
            .iter()
            .any(|run| run.text.contains('\u{00ad}'))
    );
    automatic_page.snapshot_pixels(140, 100);
    manual_page.snapshot_pixels(140, 100);
    let lines = |page: &PageRasterizer| {
        page.buffer
            .layout_runs()
            .map(|line| line.text.to_owned())
            .collect::<Vec<_>>()
    };
    assert_ne!(lines(&automatic_page), lines(&manual_page));
}

#[test]
fn automatic_hyphenation_requires_a_supported_language() {
    let automatic = [paragraph_with_language(
        "zz-ZZ",
        vec![UiNode::Text("hyphenation".to_owned())],
    )];
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
                value: "auto".to_owned(),
            },
        ],
    }];
    let mut automatic_page = PageRasterizer::new(&automatic, &stylesheet);
    let mut plain_page = PageRasterizer::new(&plain_text, &[]);

    assert_eq!(
        automatic_page.snapshot_pixels(140, 100),
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

fn paragraph_with_language(language: &str, children: Vec<UiNode>) -> UiNode {
    UiNode::Element {
        tag: "p".to_owned(),
        class: None,
        id: None,
        attributes: vec![("lang".to_owned(), Some(language.to_owned()))],
        children,
    }
}
