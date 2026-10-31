use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_order_reorders_direct_items_without_changing_nth_child_matching() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["First item", "Second item", "Third item"]
            .into_iter()
            .enumerate()
            .map(|(index, text)| UiNode::Element {
                tag: "p".to_owned(),
                class: Some(
                    match index {
                        0 => "first",
                        1 => "second",
                        _ => "third",
                    }
                    .to_owned(),
                ),
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Text(text.to_owned())],
            })
            .collect(),
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec![".container".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "display".to_owned(),
                value: "flex".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".first".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "order".to_owned(),
                value: "2".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".second".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "order".to_owned(),
                value: "-1".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".third".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "order".to_owned(),
                value: "2".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".container p:nth-child(1)".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "color".to_owned(),
                value: "red".to_owned(),
            }],
        },
    ];

    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 120);

    let lines = page
        .buffer
        .layout_runs()
        .map(|line| line.text.replace('\u{200b}', ""))
        .collect::<Vec<_>>();
    assert_eq!(lines, ["Second itemFirst itemThird item"]);
    assert!(
        page.runs
            .iter()
            .any(|run| run.text.contains('F') && run.style.color[0] > run.style.color[1])
    );

    let mut reverse_stylesheet = stylesheet.to_vec();
    reverse_stylesheet.push(StyleRule {
        selectors: vec![".container".to_owned()],
        layer: None,
        declarations: vec![StyleDeclaration {
            property: "flex-direction".to_owned(),
            value: "row-reverse".to_owned(),
        }],
    });
    let mut reverse_page = PageRasterizer::new(&nodes, &reverse_stylesheet);
    reverse_page.prepare_layout(400, 120);
    let reversed_lines = reverse_page
        .buffer
        .layout_runs()
        .map(|line| line.text.replace('\u{200b}', ""))
        .collect::<Vec<_>>();
    assert_eq!(
        reversed_lines,
        ["Third itemFirst itemSecond item"],
        "row-reverse should reverse order-modified item order while keeping source selectors"
    );
    assert!(
        reverse_page
            .runs
            .iter()
            .any(|run| { run.text.contains('F') && run.style.color[0] > run.style.color[1] })
    );
}
