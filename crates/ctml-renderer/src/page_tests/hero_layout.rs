use crate::{StyleDeclaration, StyleRule, page::PageRasterizer};
use ctml_compiler::{parse, parse_stylesheet};

#[test]
fn preview_footer_bottom_padding_stays_inside_its_flex_panel() {
    let document = parse(include_str!(
        "../../../../examples/native-app/src/main.ctml"
    ))
    .expect("showcase CTML should parse");
    let nodes = document.body.iter().map(super::ui_node).collect::<Vec<_>>();
    let css = format!(
        "{}\n{}",
        include_str!("../../../../examples/native-app/src/base.css"),
        include_str!("../../../../examples/native-app/src/main.css")
    );
    let stylesheet = parse_stylesheet(&css)
        .expect("showcase CSS should parse")
        .rules
        .into_iter()
        .map(|rule| StyleRule {
            selectors: rule.selectors,
            layer: rule.layer,
            declarations: rule
                .declarations
                .into_iter()
                .map(|declaration| StyleDeclaration {
                    property: declaration.property,
                    value: declaration.value,
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let mut page = PageRasterizer::new(&nodes, &stylesheet);
    page.prepare_layout(1920, 1024);

    let panel_id = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_sizes)
        .find(|size| size.background_layers.contains(&[34, 34, 37, 255]))
        .expect("preview panel style should be present")
        .node_id;
    let footer_id = page
        .runs
        .iter()
        .flat_map(|run| &run.style.layout_end_sizes)
        .find(|size| {
            size.border_style[0] == crate::style::BorderLineStyle::Solid
                && size.padding[0].as_deref() == Some("15px")
        })
        .expect("preview footer style should be present")
        .node_id;
    let panel_box = page
        .overflow_geometry
        .get(&panel_id)
        .expect("preview panel should have border geometry")
        .border_box;
    let footer_box = page
        .overflow_geometry
        .get(&footer_id)
        .expect("preview footer should have border geometry")
        .border_box;
    let bottom_inset = panel_box.bottom - footer_box.bottom;
    assert!(
        (-0.5..=2.5).contains(&bottom_inset),
        "footer should fit flush inside the preview panel, got {bottom_inset}px bottom inset"
    );
}
