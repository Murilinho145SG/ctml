use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

fn red_border_pixels(decoration_break: &str) -> usize {
    let text = "A long inline span wraps onto several formatted lines";
    let nodes = [UiNode::Element {
        tag: "div".to_owned(),
        class: None,
        id: None,
        attributes: Vec::new(),
        children: vec![UiNode::Element {
            tag: "span".to_owned(),
            class: Some("outlined".to_owned()),
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text(text.to_owned())],
        }],
    }];
    let stylesheet = [
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![("width", "105px")]
                .into_iter()
                .map(|(property, value)| StyleDeclaration {
                    property: property.to_owned(),
                    value: value.to_owned(),
                })
                .collect(),
        },
        StyleRule {
            selectors: vec![".outlined".to_owned()],
            layer: None,
            declarations: vec![
                ("border", "2px solid red"),
                ("box-decoration-break", decoration_break),
            ]
            .into_iter()
            .map(|(property, value)| StyleDeclaration {
                property: property.to_owned(),
                value: value.to_owned(),
            })
            .collect(),
        },
    ];
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    let pixels = page.snapshot_pixels(180, 180);
    pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 200 && pixel[1] < 80 && pixel[2] < 80)
        .count()
}

#[test]
fn clone_repeats_inline_box_edge_borders_on_each_line_fragment() {
    let sliced = red_border_pixels("slice");
    let cloned = red_border_pixels("clone");
    assert!(
        sliced > 20,
        "slice should paint the horizontal edges and outer inline edges"
    );
    assert!(
        cloned > sliced,
        "clone should repeat start and end borders on each line fragment: {cloned} vs {sliced}"
    );
}
