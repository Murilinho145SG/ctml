use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

#[test]
fn fixed_height_flex_row_reserves_its_block_size_before_following_content() {
    let nodes = [
        UiNode::Element {
            tag: "header".to_owned(),
            class: Some("topbar".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: ["Brand", "Status"]
                .into_iter()
                .map(|text| UiNode::Element {
                    tag: "span".to_owned(),
                    class: None,
                    id: None,
                    attributes: Vec::new(),
                    children: vec![UiNode::Text(text.to_owned())],
                })
                .collect(),
        },
        UiNode::Element {
            tag: "p".to_owned(),
            class: Some("following".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Following content".to_owned())],
        },
    ];
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
            selectors: vec![".topbar".to_owned()],
            layer: None,
            declarations: [
                ("display", "flex"),
                ("height", "56px"),
                ("justify-content", "space-between"),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        },
        StyleRule {
            selectors: vec![".following".to_owned()],
            layer: None,
            declarations: vec![StyleDeclaration {
                property: "margin-top".to_owned(),
                value: "22px".to_owned(),
            }],
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(400, 160);

    let topbar_line = page
        .layout_runs()
        .find(|line| line.text.contains("Brand"))
        .expect("flex items should render");
    let following_line = page
        .layout_runs()
        .find(|line| line.text.contains("Following content"))
        .expect("following block should render");
    assert!(
        following_line.line_top >= topbar_line.line_top + 77.0,
        "following content should start after the 56px flex row and 22px margin; row y={}, following y={}",
        topbar_line.line_top,
        following_line.line_top
    );
}
