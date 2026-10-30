mod generated {
    include!(concat!(env!("OUT_DIR"), "/window_config.rs"));
}

fn main() -> Result<(), ctml::WindowError> {
    ctml::Window::create(ctml::WindowConfig {
        title: generated::WINDOW_TITLE.to_owned(),
        width: generated::WINDOW_WIDTH,
        height: generated::WINDOW_HEIGHT,
        body: generated::body(),
        stylesheet: generated::stylesheet(),
        fonts: generated::fonts(),
        color_profiles: generated::color_profiles(),
    })
}
