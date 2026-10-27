use std::{env, fs, io, path::PathBuf};

use ctml_compiler::{AttributeValue, CssRule, Element, Node, parse, parse_stylesheet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_path = "src/main.ctml";
    println!("cargo:rerun-if-changed={source_path}");

    let source = fs::read_to_string(source_path)?;
    let document = parse(&source)?;
    let stylesheet = match document.stylesheet() {
        Some(element) => {
            let relative_path = string_attribute(element, "href")?
                .ok_or_else(|| invalid_data("<stylesheet> requires an href attribute"))?;
            let relative_path = PathBuf::from(relative_path);
            if relative_path.is_absolute()
                || relative_path
                    .components()
                    .any(|component| !matches!(component, std::path::Component::Normal(_)))
            {
                return Err(
                    invalid_data("stylesheet href must be a relative path inside src/").into(),
                );
            }
            let stylesheet_path = PathBuf::from("src").join(relative_path);
            println!("cargo:rerun-if-changed={}", stylesheet_path.display());
            let source = fs::read_to_string(stylesheet_path)?;
            parse_stylesheet(&source).map_err(invalid_data)?.rules
        }
        None => Vec::new(),
    };
    let application = document
        .application()
        .ok_or_else(|| invalid_data("<headers> must contain an <application> element"))?;

    let title = string_attribute(application, "name")?.unwrap_or("CTML Application");
    let width = dimension_attribute(application, "width", 800)?;
    let height = dimension_attribute(application, "height", 600)?;
    let body = rust_nodes(&document.body)?;
    let stylesheet = rust_stylesheet(&stylesheet);
    let generated_source = format!(
        "pub const WINDOW_TITLE: &str = {title:?};\npub const WINDOW_WIDTH: u32 = {width};\npub const WINDOW_HEIGHT: u32 = {height};\npub fn body() -> Vec<ctml::UiNode> {{ vec![{body}] }}\npub fn stylesheet() -> Vec<ctml::StyleRule> {{ vec![{stylesheet}] }}\n"
    );

    let output_path = PathBuf::from(
        env::var_os("OUT_DIR")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo did not set OUT_DIR"))?,
    )
    .join("window_config.rs");
    fs::write(output_path, generated_source)?;
    Ok(())
}

fn rust_nodes(nodes: &[Node]) -> Result<String, io::Error> {
    nodes
        .iter()
        .map(rust_node)
        .collect::<Result<Vec<_>, _>>()
        .map(|nodes| nodes.join(", "))
}

fn rust_node(node: &Node) -> Result<String, io::Error> {
    match node {
        Node::Text(text) => Ok(format!("ctml::UiNode::Text({text:?}.to_owned())")),
        Node::Element(element) => {
            let class = string_attribute(element, "class")?;
            let id = string_attribute(element, "id")?;
            Ok(format!(
                "ctml::UiNode::Element {{ tag: {:?}.to_owned(), class: {}, id: {}, children: vec![{}] }}",
                element.name,
                option_string(class),
                option_string(id),
                rust_nodes(&element.children)?
            ))
        }
    }
}

fn rust_stylesheet(rules: &[CssRule]) -> String {
    rules
        .iter()
        .map(|rule| {
            let selectors = rule
                .selectors
                .iter()
                .map(|selector| format!("{selector:?}.to_owned()"))
                .collect::<Vec<_>>()
                .join(", ");
            let declarations = rule
                .declarations
                .iter()
                .map(|declaration| {
                    format!(
                        "ctml::StyleDeclaration {{ property: {:?}.to_owned(), value: {:?}.to_owned() }}",
                        declaration.property, declaration.value
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "ctml::StyleRule {{ selectors: vec![{selectors}], declarations: vec![{declarations}] }}"
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn option_string(value: Option<&str>) -> String {
    value.map_or_else(
        || "None".to_owned(),
        |value| format!("Some({value:?}.to_owned())"),
    )
}

fn string_attribute<'a>(element: &'a Element, name: &str) -> Result<Option<&'a str>, io::Error> {
    match element.attribute(name) {
        Some(AttributeValue::String(value)) => Ok(Some(value)),
        Some(AttributeValue::Boolean) => Err(invalid_data(format!(
            "attribute '{name}' requires a quoted string"
        ))),
        None => Ok(None),
    }
}

fn dimension_attribute(element: &Element, name: &str, default: u32) -> Result<u32, io::Error> {
    let Some(value) = string_attribute(element, name)? else {
        return Ok(default);
    };
    let dimension = value
        .parse::<u32>()
        .map_err(|_| invalid_data(format!("attribute '{name}' must be an unsigned integer")))?;
    if dimension == 0 {
        return Err(invalid_data(format!(
            "attribute '{name}' must be greater than zero"
        )));
    }
    Ok(dimension)
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
