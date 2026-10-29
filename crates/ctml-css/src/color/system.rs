pub(super) fn parse_system_color(value: &str) -> Option<[u8; 3]> {
    let color = match value.to_ascii_lowercase().as_str() {
        "accentcolor" => [0, 120, 212],
        "accentcolortext" => [255, 255, 255],
        "activetext" => [192, 0, 0],
        "buttonborder" | "activeborder" | "inactiveborder" | "threeddarkshadow"
        | "threedhighlight" | "threedlightshadow" | "threedshadow" | "windowframe" => {
            [118, 118, 118]
        }
        "buttonface" | "threedface" | "buttonhighlight" | "buttonshadow" => [240, 240, 240],
        "buttontext" | "canvastext" | "fieldtext" | "marktext" | "menutext" | "infotext"
        | "windowtext" => [0, 0, 0],
        "canvas" | "field" | "activecaption" | "appworkspace" | "background"
        | "inactivecaption" | "infobackground" | "menu" | "scrollbar" | "window" => [255, 255, 255],
        "captiontext" => [0, 0, 0],
        "graytext" | "inactivecaptiontext" => [109, 109, 109],
        "highlight" | "selecteditem" => [0, 76, 192],
        "highlighttext" | "selecteditemtext" => [255, 255, 255],
        "linktext" => [0, 0, 238],
        "mark" => [255, 255, 0],
        "visitedtext" => [85, 26, 139],
        _ => return None,
    };
    Some(color)
}
