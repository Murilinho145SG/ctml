use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn flex_row_items_shrink_for_their_margins_and_keep_the_inner_gap() {
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: Some("container".to_owned()),
        id: None,
        attributes: Vec::new(),
        children: ["First", "Second", "Third"]
            .into_iter()
            .enumerate()
            .map(|(index, text)| UiNode::Element {
                tag: "div".to_owned(),
                class: Some(if index == 2 { "item last" } else { "item" }.to_owned()),
                id: None,
                attributes: Vec::new(),
                children: vec![UiNode::Element {
                    tag: "p".to_owned(),
                    class: None,
                    id: None,
                    attributes: Vec::new(),
                    children: vec![UiNode::Text(text.to_owned())],
                }],
            })
            .collect(),
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "margin".to_owned(),
                value: "0".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["*".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "box-sizing".to_owned(),
                value: "border-box".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec!["p".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "margin".to_owned(),
                value: "0".to_owned(),
            }],
        },
        StyleRule {
            selectors: vec![".container".to_owned()],
            layer: None,
            declarations: [
                ("display", "flex"),
                ("justify-content", "space-between"),
                ("width", "600px"),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        },
        StyleRule {
            selectors: vec![".item".to_owned()],
            layer: None,
            declarations: [
                ("width", "33.33%"),
                ("margin-right", "28px"),
                ("padding-right", "28px"),
                ("border-right", "1px solid red"),
                ("height", "40px"),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        },
        StyleRule {
            selectors: vec![".last".to_owned()],
            layer: None,
            declarations: [("margin-right", "0"), ("border-right", "0")]
                .into_iter()
                .map(|(property, value)| StyleDeclaration {
                    property: property.to_owned(),
                    value: value.to_owned(),
                })
                .collect(),
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(620, 80);
    let mut item_ids = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_sizes)
        .filter(|size| size.width.as_deref() == Some("33.33%"))
        .map(|size| size.node_id)
        .collect::<Vec<_>>();
    item_ids.sort_unstable();
    item_ids.dedup();
    item_ids
        .sort_by(|left, right| page.flex_item_box_x[left].total_cmp(&page.flex_item_box_x[right]));

    assert_eq!(item_ids.len(), 3);
    let widths = item_ids
        .iter()
        .map(|id| page.flex_item_box_width[id])
        .collect::<Vec<_>>();
    assert!(widths.iter().all(|width| (*width - 181.33).abs() < 1.0));
    let positions = item_ids
        .iter()
        .map(|id| page.flex_item_box_x[id])
        .collect::<Vec<_>>();
    for index in 0..2 {
        let gap = positions[index + 1] - positions[index] - widths[index];
        assert!(
            (gap - 28.0).abs() < 1.0,
            "the flex item margin should remain between its border and the next item, got {gap}px"
        );
    }
}
