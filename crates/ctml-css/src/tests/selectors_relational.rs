use super::*;

#[test]
fn matches_has_with_descendant_child_and_sibling_relative_selectors() {
    let body = identity("body", None);
    let main = identity("main", None);
    let paragraph = identity("p", Some("active"));
    let div = identity("div", Some("warning"));
    let emphasis = identity("em", None);
    let main_position = position(1, 1, 1);
    let body_position = position(0, 1, 1);
    let body_ancestor = tree_ancestor(body.clone(), body_position, vec![]);
    let main_ancestor = tree_ancestor(main.clone(), main_position, vec![]);
    let paragraph_position = position(2, 1, 2);
    let div_position = position(3, 2, 2);
    let emphasis_position = position(4, 1, 1);
    let tree = vec![
        tree_element(0, None, body.clone(), body_position, vec![]),
        tree_element(
            1,
            Some(0),
            main.clone(),
            main_position,
            vec![body_ancestor.clone()],
        ),
        tree_element(
            2,
            Some(1),
            paragraph.clone(),
            paragraph_position,
            vec![body_ancestor.clone(), main_ancestor.clone()],
        ),
        tree_element(
            3,
            Some(1),
            div.clone(),
            div_position,
            vec![body_ancestor.clone(), main_ancestor.clone()],
        ),
        tree_element(
            4,
            Some(3),
            emphasis.clone(),
            emphasis_position,
            vec![
                body_ancestor.clone(),
                main_ancestor.clone(),
                tree_ancestor(
                    div.clone(),
                    div_position,
                    vec![sibling(2, paragraph.clone(), paragraph_position)],
                ),
            ],
        ),
    ];
    for selector in ["main:has(> p.active)", "main:has(.warning em)"] {
        assert!(
            selector_matches_with_selector_tree_context(
                selector,
                &main,
                std::slice::from_ref(&body),
                &[],
                &[],
                &[],
                &[vec![]],
                main_position,
                &[body_position],
                &tree,
            ),
            "{selector}"
        );
    }
    for selector in [
        "div:nth-child(2 of .active, .warning)",
        "div:nth-last-child(1 of .warning)",
    ] {
        assert!(
            selector_matches_with_selector_tree_context(
                selector,
                &div,
                &[body.clone(), main.clone()],
                &[],
                &[],
                &[sibling(2, paragraph.clone(), paragraph_position)],
                &[vec![], vec![]],
                div_position,
                &[body_position, main_position],
                &tree,
            ),
            "{selector}"
        );
    }
    assert!(!selector_matches_with_selector_tree_context(
        "div:nth-child(2 of .active)",
        &div,
        &[body.clone(), main.clone()],
        &[],
        &[],
        &[sibling(2, paragraph.clone(), paragraph_position)],
        &[vec![], vec![]],
        div_position,
        &[body_position, main_position],
        &tree,
    ));
    assert!(!selector_matches(
        "main:has(> p.active)",
        &main,
        &[body.clone()]
    ));

    let heading = identity("h1", None);
    let notice = identity("p", Some("notice"));
    let heading_position = position(5, 1, 2);
    let notice_position = position(6, 2, 2);
    let root_ancestor = tree_ancestor(body.clone(), body_position, vec![]);
    let heading_node = tree_element(
        5,
        Some(0),
        heading.clone(),
        heading_position,
        vec![root_ancestor.clone()],
    );
    let notice_node = SelectorTreeElement {
        node_id: 6,
        parent_id: Some(0),
        element: notice.clone(),
        attributes: vec![("class".to_owned(), Some("notice".to_owned()))],
        position: notice_position,
        siblings_before: vec![sibling(5, heading.clone(), heading_position)],
        ancestors: vec![root_ancestor],
    };
    let tree = vec![
        tree_element(0, None, body.clone(), body_position, vec![]),
        heading_node,
        notice_node,
    ];
    assert!(selector_matches_with_selector_tree_context(
        "h1:has(+ p.notice)",
        &heading,
        &[],
        &[],
        &[],
        &[],
        &[],
        heading_position,
        &[],
        &tree,
    ));
    assert!(selector_matches_with_selector_tree_context(
        "h1:has(~ p.notice)",
        &heading,
        &[],
        &[],
        &[],
        &[],
        &[],
        heading_position,
        &[],
        &tree,
    ));
}
