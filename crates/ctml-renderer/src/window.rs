use std::{sync::Arc, time::Duration, time::Instant};

use super::renderer::Renderer;
use crate::{StyleRule, UiNode};

use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    error::{EventLoopError, OsError},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{CursorIcon, Window as NativeWindow, WindowId},
};

const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(500);
const DOUBLE_CLICK_RADIUS_SQUARED: f64 = 25.0;

#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    /// Content compiled from the document's `<body>` section.
    pub body: Vec<UiNode>,
    /// CSS rules compiled from the linked `.css` stylesheet.
    pub stylesheet: Vec<StyleRule>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "CTML Application".to_owned(),
            width: 800,
            height: 600,
            body: Vec::new(),
            stylesheet: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum WindowError {
    #[error("window width and height must be greater than zero")]
    InvalidSize,
    #[error(transparent)]
    EventLoop(#[from] EventLoopError),
    #[error(transparent)]
    NativeWindow(#[from] OsError),
    #[error("no compatible graphics adapter was found: {0}")]
    Adapter(#[from] wgpu::RequestAdapterError),
    #[error("failed to create the graphics surface: {0}")]
    Surface(#[from] wgpu::CreateSurfaceError),
    #[error("failed to create the graphics device: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
    #[error("the graphics adapter does not support this window surface")]
    UnsupportedSurface,
    #[error("wgpu reported a validation error while acquiring the window surface")]
    SurfaceValidation,
}

/// A native application window backed by winit and wgpu.
pub struct Window;

impl Window {
    /// Creates a native window and runs its event loop until the window closes.
    pub fn create(config: WindowConfig) -> Result<(), WindowError> {
        if config.width == 0 || config.height == 0 {
            return Err(WindowError::InvalidSize);
        }

        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);

        let mut app = WindowApplication::new(config);
        event_loop.run_app(&mut app)?;

        match app.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

struct WindowApplication {
    config: WindowConfig,
    window: Option<Arc<NativeWindow>>,
    renderer: Option<Renderer>,
    error: Option<WindowError>,
    cursor_position: PhysicalPosition<f64>,
    left_button_down: bool,
    click_start: PhysicalPosition<f64>,
    dragged_during_click: bool,
    double_click_active: bool,
    last_click: Option<(Instant, PhysicalPosition<f64>)>,
}

impl WindowApplication {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            window: None,
            renderer: None,
            error: None,
            cursor_position: PhysicalPosition::new(0.0, 0.0),
            left_button_down: false,
            click_start: PhysicalPosition::new(0.0, 0.0),
            dragged_during_click: false,
            double_click_active: false,
            last_click: None,
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: WindowError) {
        self.error = Some(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for WindowApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = NativeWindow::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(PhysicalSize::new(self.config.width, self.config.height));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.fail(event_loop, error.into());
                return;
            }
        };

        let renderer = match pollster::block_on(Renderer::new(
            window.clone(),
            event_loop.owned_display_handle(),
            self.config.width,
            self.config.height,
            &self.config.body,
            &self.config.stylesheet,
        )) {
            Ok(renderer) => renderer,
            Err(error) => {
                self.fail(event_loop, error);
                return;
            }
        };

        window.request_redraw();
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.left_button_down
                    && !self.dragged_during_click
                    && !positions_are_close(self.click_start, position)
                {
                    self.dragged_during_click = true;
                }
                self.cursor_position = position;
                if let Some(renderer) = self.renderer.as_mut() {
                    let (cursor, cursor_visible, selection_changed) =
                        renderer.pointer_moved(position);
                    window.set_cursor(cursor);
                    window.set_cursor_visible(cursor_visible);
                    if selection_changed {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                window.set_cursor(CursorIcon::Default);
                window.set_cursor_visible(true);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    match state {
                        winit::event::ElementState::Pressed => {
                            let now = Instant::now();
                            let is_double_click =
                                self.last_click
                                    .take()
                                    .is_some_and(|(last_time, last_position)| {
                                        now.duration_since(last_time) <= DOUBLE_CLICK_WINDOW
                                            && positions_are_close(
                                                last_position,
                                                self.cursor_position,
                                            )
                                    });
                            self.left_button_down = true;
                            self.click_start = self.cursor_position;
                            self.dragged_during_click = false;
                            self.double_click_active = is_double_click;
                            if let Some(renderer) = self.renderer.as_mut() {
                                let changed = if is_double_click {
                                    renderer.select_word_at(self.cursor_position)
                                } else {
                                    renderer.mouse_input(state, button, self.cursor_position)
                                };
                                if changed {
                                    window.request_redraw();
                                }
                            }
                        }
                        winit::event::ElementState::Released => {
                            if let Some(renderer) = self.renderer.as_mut() {
                                renderer.mouse_input(state, button, self.cursor_position);
                            }
                            if !self.dragged_during_click && !self.double_click_active {
                                self.last_click = Some((Instant::now(), self.cursor_position));
                            } else {
                                self.last_click = None;
                            }
                            self.left_button_down = false;
                            self.double_click_active = false;
                        }
                    }
                } else if let Some(renderer) = self.renderer.as_mut() {
                    renderer.mouse_input(state, button, self.cursor_position);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut()
                    && let Err(error) = renderer.render(window)
                {
                    self.fail(event_loop, error);
                }
            }
            _ => {}
        }
    }
}

fn positions_are_close(first: PhysicalPosition<f64>, second: PhysicalPosition<f64>) -> bool {
    let delta_x = first.x - second.x;
    let delta_y = first.y - second.y;
    delta_x * delta_x + delta_y * delta_y <= DOUBLE_CLICK_RADIUS_SQUARED
}
