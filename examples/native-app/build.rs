use std::{env, fs, io, path::PathBuf};

use ctml_compiler::{
    AttributeValue, CssFontSource, CssRule, Element, Node, load_local_resource, parse,
    parse_font_face_sources, parse_stylesheet_file,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_path = "src/main.ctml";
    println!("cargo:rerun-if-changed={source_path}");

    let source = fs::read_to_string(source_path)?;
    let document = parse(&source)?;
    let mut stylesheet = Vec::new();
    let mut fonts = Vec::new();
    let mut layers = Vec::new();
    for element in document.stylesheets() {
        let relative_path = string_attribute(element, "href")?
            .ok_or_else(|| invalid_data("<stylesheet> requires an href attribute"))?;
        let relative_path = PathBuf::from(relative_path);
        if relative_path.is_absolute()
            || relative_path
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(invalid_data("stylesheet href must be a relative path inside src/").into());
        }
        let source_root = PathBuf::from("src");
        let stylesheet_path = source_root.join(relative_path);
        let (parsed, dependencies) =
            parse_stylesheet_file(&stylesheet_path, &source_root).map_err(invalid_data)?;
        for dependency in dependencies {
            println!("cargo:rerun-if-changed={}", dependency.display());
        }
        for face in &parsed.font_faces {
            let path = face
                .stylesheet_path
                .as_ref()
                .ok_or_else(|| invalid_data("font-face has no source stylesheet"))?;
            let candidates = parse_font_face_sources(&face.source)
                .ok_or_else(|| invalid_data("invalid @font-face src descriptor"))?;
            let mut sources = Vec::new();
            for candidate in candidates {
                match candidate {
                    CssFontSource::Local(name) => sources.push((Some(name), None)),
                    CssFontSource::Url(url) => {
                        let resource =
                            load_local_resource(&url, path, &source_root).map_err(invalid_data)?;
                        println!("cargo:rerun-if-changed={}", resource.path.display());
                        sources.push((None, Some(resource.path)));
                    }
                }
            }
            fonts.push((
                face.family.clone(),
                sources,
                face.weight.clone(),
                face.weight_range,
                face.style.clone(),
                face.style_range.clone(),
                face.stretch.clone(),
            ));
        }
        for layer in &parsed.layers {
            if !layers.contains(layer) {
                let insertion = layer
                    .rsplit_once('.')
                    .and_then(|(parent, _)| layers.iter().position(|known| known == parent))
                    .unwrap_or(layers.len());
                layers.insert(insertion, layer.clone());
            }
        }
        for mut rule in parsed.rules {
            rule.layer = rule.layer.map(|index| {
                let name = &parsed.layers[index];
                layers
                    .iter()
                    .position(|layer| layer == name)
                    .expect("merged cascade layer should be present")
            });
            stylesheet.push(rule);
        }
    }
    let application = document
        .application()
        .ok_or_else(|| invalid_data("<headers> must contain an <application> element"))?;

    let title = string_attribute(application, "name")?.unwrap_or("CTML Application");
    let width = dimension_attribute(application, "width", 800)?;
    let height = dimension_attribute(application, "height", 600)?;
    let body = rust_nodes(&document.body)?;
    let stylesheet = rust_stylesheet(&stylesheet);
    let fonts = rust_fonts(&fonts);
    let generated_source = format!(
        "pub const WINDOW_TITLE: &str = {title:?};\npub const WINDOW_WIDTH: u32 = {width};\npub const WINDOW_HEIGHT: u32 = {height};\npub fn body() -> Vec<ctml::UiNode> {{ vec![{body}] }}\npub fn stylesheet() -> Vec<ctml::StyleRule> {{ vec![{stylesheet}] }}\npub fn fonts() -> Vec<ctml::FontFace> {{ vec![{fonts}] }}\n"
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
                "ctml::UiNode::Element {{ tag: {:?}.to_owned(), class: {}, id: {}, attributes: vec![{}], children: vec![{}] }}",
                element.name,
                option_string(class),
                option_string(id),
                rust_attributes(element),
                rust_nodes(&element.children)?
            ))
        }
    }
}

fn rust_attributes(element: &Element) -> String {
    element
        .attributes
        .iter()
        .map(|attribute| {
            let value = attribute.value.as_str().map_or_else(
                || "None".to_owned(),
                |value| format!("Some({value:?}.to_owned())"),
            );
            format!("({:?}.to_owned(), {value})", attribute.name)
        })
        .collect::<Vec<_>>()
        .join(", ")
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
                "ctml::StyleRule {{ selectors: vec![{selectors}], declarations: vec![{declarations}], layer: {} }}",
                rule.layer.map_or_else(|| "None".to_owned(), |layer| format!("Some({layer})"))
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn rust_fonts(
    fonts: &[(
        String,
        Vec<(Option<String>, Option<PathBuf>)>,
        Option<String>,
        Option<(u16, u16)>,
        Option<String>,
        Option<(String, String)>,
        Option<String>,
    )],
) -> String {
    fonts.iter().map(|(family, sources, weight, weight_range, style, style_range, stretch)| {
        let weight = weight.as_deref().and_then(parse_font_weight)
            .map_or_else(|| "None".to_owned(), |weight| format!("Some({weight})"));
        let weight_range = weight_range.map_or_else(|| "None".to_owned(), |(start, end)| format!("Some(({start}, {end}))"));
        let style = style.as_deref().filter(|value| is_font_style(value))
            .map_or_else(|| "None".to_owned(), |value| format!("Some({value:?}.to_owned())"));
        let style_range = style_range.as_ref().map_or_else(|| "None".to_owned(), |(start, end)| format!("Some(({start:?}.to_owned(), {end:?}.to_owned()))"));
        let stretch = stretch.as_deref().filter(|value| is_font_stretch(value))
            .map_or_else(|| "None".to_owned(), |value| format!("Some({value:?}.to_owned())"));
        let sources = sources.iter().map(|(local, path)| match (local, path) {
            (Some(name), None) => format!("ctml::FontFaceSource::Local({name:?}.to_owned())"),
            (None, Some(path)) => format!(
                "ctml::FontFaceSource::Embedded(include_bytes!({:?}).to_vec())",
                path.to_str().expect("canonical resource path must be UTF-8"),
            ),
            _ => unreachable!("font source candidate must have exactly one variant"),
        }).collect::<Vec<_>>().join(", ");
        format!(
            "ctml::FontFace {{ family: {family:?}.to_owned(), sources: vec![{sources}], weight: {weight}, weight_range: {weight_range}, style: {style}, style_range: {style_range}, stretch: {stretch} }}"
        )
    }).collect::<Vec<_>>().join(", ")
}

fn parse_font_weight(value: &str) -> Option<u16> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(400),
        "bold" => Some(700),
        value => value
            .parse()
            .ok()
            .filter(|weight: &u16| (1..=1000).contains(weight)),
    }
}

fn is_font_style(value: &str) -> bool {
    let mut parts = value.split_ascii_whitespace();
    match parts.next().map(str::to_ascii_lowercase).as_deref() {
        Some("normal" | "italic") => parts.next().is_none(),
        Some("oblique") => {
            let angles = parts.collect::<Vec<_>>();
            angles.is_empty()
                || ((angles.len() == 1 || angles.len() == 2)
                    && angles.iter().all(|angle| {
                        parse_css_angle(angle).is_some_and(|degrees| {
                            degrees.is_finite() && (-90.0..=90.0).contains(&degrees)
                        })
                    }))
        }
        _ => false,
    }
}

fn parse_css_angle(value: &str) -> Option<f32> {
    if value == "0" {
        return Some(0.0);
    }
    let value = value.to_ascii_lowercase();
    let (number, unit) = ["deg", "grad", "rad", "turn"]
        .into_iter()
        .find_map(|unit| value.strip_suffix(unit).map(|number| (number, unit)))?;
    let number = number.parse::<f32>().ok()?;
    let degrees = match unit {
        "deg" => number,
        "grad" => number * 0.9,
        "rad" => number.to_degrees(),
        "turn" => number * 360.0,
        _ => return None,
    };
    degrees.is_finite().then_some(degrees)
}

fn is_font_stretch(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "ultra-condensed"
            | "extra-condensed"
            | "condensed"
            | "semi-condensed"
            | "normal"
            | "semi-expanded"
            | "expanded"
            | "extra-expanded"
            | "ultra-expanded"
    ) || value.trim().strip_suffix('%').is_some_and(|percent| {
        percent
            .trim()
            .parse::<f32>()
            .is_ok_and(|value| value.is_finite() && value > 0.0)
    })
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
