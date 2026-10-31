pub(super) fn parse_system_color(value: &str, dark_scheme: bool) -> Option<[u8; 3]> {
    let keyword = value.to_ascii_lowercase();
    if dark_scheme {
        let dark = match keyword.as_str() {
            "accentcolor" => Some([0, 120, 212]),
            "accentcolortext" | "buttontext" | "canvastext" | "fieldtext" | "menutext"
            | "infotext" | "windowtext" | "highlighttext" | "selecteditemtext" => {
                Some([245, 245, 245])
            }
            "activetext" => Some([255, 128, 128]),
            "buttonborder" | "activeborder" | "inactiveborder" | "threeddarkshadow"
            | "threedhighlight" | "threedlightshadow" | "threedshadow" | "windowframe" => {
                Some([128, 128, 128])
            }
            "buttonface" | "threedface" | "buttonhighlight" | "buttonshadow" => Some([48, 48, 48]),
            "canvas" | "field" | "activecaption" | "appworkspace" | "background"
            | "inactivecaption" | "infobackground" | "menu" | "scrollbar" | "window" => {
                Some([24, 24, 24])
            }
            "captiontext" => Some([245, 245, 245]),
            "graytext" | "inactivecaptiontext" => Some([160, 160, 160]),
            "highlight" | "selecteditem" => Some([10, 132, 255]),
            "linktext" => Some([138, 180, 248]),
            "mark" => Some([255, 255, 0]),
            "marktext" => Some([0, 0, 0]),
            "visitedtext" => Some([197, 138, 249]),
            _ => None,
        };
        if dark.is_some() {
            return dark;
        }
    }
    let color = match keyword.as_str() {
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
