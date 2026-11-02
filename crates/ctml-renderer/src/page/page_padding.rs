use std::cell::Cell;

thread_local! {
    static CURRENT_PAGE_PADDING: Cell<f32> = const { Cell::new(super::PAGE_PADDING) };
}

pub fn current() -> f32 {
    CURRENT_PAGE_PADDING.with(Cell::get)
}

pub fn update(padding: f32) {
    CURRENT_PAGE_PADDING.with(|current| current.set(padding));
}

#[cfg(test)]
mod tests {
    use crate::{StyleDeclaration, StyleRule, UiNode, page::PageRasterizer};

    #[test]
    fn explicit_zero_body_margin_uses_the_full_viewport() {
        let pixels = render_block(Some("0"));
        assert_eq!(pixel(&pixels, 40, 2, 2), [255, 0, 0]);
        assert_ne!(pixel(&pixels, 40, 14, 14), [255, 0, 0]);
    }

    #[test]
    fn unspecified_body_margin_keeps_the_default_page_gutter() {
        let pixels = render_block(None);
        assert_ne!(pixel(&pixels, 40, 2, 2), [255, 0, 0]);
        assert_eq!(pixel(&pixels, 40, 10, 10), [255, 0, 0]);
    }

    fn render_block(body_margin: Option<&str>) -> Vec<u8> {
        let nodes = [UiNode::Element {
            tag: "div".to_owned(),
            class: None,
            id: None,
            attributes: Vec::new(),
            children: Vec::new(),
        }];
        let mut stylesheet = Vec::new();
        if let Some(margin) = body_margin {
            stylesheet.push(StyleRule {
                selectors: vec!["body".to_owned()],
                layer: None,
                declarations: vec![StyleDeclaration {
                    property: "margin".to_owned(),
                    value: margin.to_owned(),
                }],
            });
        }
        stylesheet.push(StyleRule {
            selectors: vec!["div".to_owned()],
            layer: None,
            declarations: vec![
                StyleDeclaration {
                    property: "width".to_owned(),
                    value: "12px".to_owned(),
                },
                StyleDeclaration {
                    property: "height".to_owned(),
                    value: "12px".to_owned(),
                },
                StyleDeclaration {
                    property: "background-color".to_owned(),
                    value: "red".to_owned(),
                },
            ],
        });
        PageRasterizer::new(&nodes, &stylesheet).snapshot_pixels(40, 40)
    }

    fn pixel(pixels: &[u8], width: usize, x: usize, y: usize) -> [u8; 3] {
        let index = (y * width + x) * 4;
        pixels[index..index + 3]
            .try_into()
            .expect("pixel has three color channels")
    }
}
