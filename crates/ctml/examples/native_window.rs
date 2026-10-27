use ctml::{Window, WindowConfig};

fn main() -> Result<(), ctml::WindowError> {
    Window::create(WindowConfig {
        title: "CTML Native Window".to_owned(),
        ..WindowConfig::default()
    })
}
