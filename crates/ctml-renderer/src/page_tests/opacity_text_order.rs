use super::super::PageRasterizer;
use crate::{StyleDeclaration, StyleRule, UiNode};

fn declaration(property: &str, value: &str) -> StyleDeclaration {
    StyleDeclaration {
        property: property.to_owned(),
        value: value.to_owned(),
    }
}

fn render(opacity: &str) -> Vec<u8> {
    let nodes = [
        UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: vec![UiNode::Text("Group".to_owned())],
        },
        UiNode::Text("Later".to_owned()),
    ];
    let stylesheet = [
        StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![
                declaration("width", "90px"),
                declaration("height", "40px"),
                declaration("margin-bottom", "-20px"),
                declaration("background-color", "red"),
                declaration("color", "transparent"),
                declaration("box-shadow", "0 35px 0 0 red"),
                declaration("opacity", opacity),
            ],
        },
        StyleRule {
            selectors: vec!["body".to_owned()],
            layer: None,
            declarations: vec![
                declaration("color", "blue"),
                declaration("font-size", "32px"),
            ],
        },
    ];
    PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(180, 120)
}

#[test]
fn later_root_text_paints_after_an_earlier_opacity_group() {
    let without_group = render("0");
    let translucent_group = render("0.5");
    let with_opaque_group = render("1");
    let bounds = |pixels: &[u8], color: [u8; 3]| {
        pixels
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, pixel)| pixel[..3] == color)
            .map(|(index, _)| (index % 180, index / 180))
            .fold(None, |bounds, (x, y)| {
                Some(bounds.map_or(
                    (x, y, x, y),
                    |(l, t, r, b): (usize, usize, usize, usize)| {
                        (l.min(x), t.min(y), r.max(x), b.max(y))
                    },
                ))
            })
    };
    let red_bounds = bounds(&with_opaque_group, [255, 0, 0]).expect("red box should be visible");
    let blue_bounds = bounds(&without_group, [0, 0, 255]).expect("blue text should be visible");
    let overlapping_bounds = red_bounds.0 <= blue_bounds.2
        && blue_bounds.0 <= red_bounds.2
        && red_bounds.1 <= blue_bounds.3
        && blue_bounds.1 <= red_bounds.3;

    assert!(
        overlapping_bounds,
        "expected later blue text bounds to overlap the earlier red paint"
    );
    for (plain, translucent) in without_group
        .chunks_exact(4)
        .zip(translucent_group.chunks_exact(4))
    {
        if plain[..3] == [0, 0, 255] {
            assert_eq!(
                plain, translucent,
                "later opaque text should cover the earlier opacity group"
            );
        }
    }
}
