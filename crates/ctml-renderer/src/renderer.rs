use std::{sync::Arc, time::Instant};

use super::window::WindowError;
use crate::{CssColorProfile, FontFace, StyleRule, UiNode, page::PageRasterizer};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
    window::{CursorIcon, Window as NativeWindow},
};

pub(super) struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    page: PageRasterizer,
    page_texture: wgpu::Texture,
    page_layout: wgpu::BindGroupLayout,
    page_sampler: wgpu::Sampler,
    page_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub(super) async fn new(
        window: Arc<NativeWindow>,
        display_handle: winit::event_loop::OwnedDisplayHandle,
        width: u32,
        height: u32,
        body: &[UiNode],
        stylesheet: &[StyleRule],
        fonts: &[FontFace],
        color_profiles: &[CssColorProfile],
    ) -> Result<Self, WindowError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display_handle),
        ));
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("ctml-device"),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await?;
        let config = surface
            .get_default_config(&adapter, width, height)
            .ok_or(WindowError::UnsupportedSurface)?;
        surface.configure(&device, &config);

        let mut page = PageRasterizer::new_with_fonts_and_color_profiles(
            body,
            stylesheet,
            fonts,
            color_profiles,
        );
        let (page_texture, page_layout, page_sampler, page_bind_group, pipeline) =
            create_page_renderer(&device, &queue, &config, &mut page)?;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            page,
            page_texture,
            page_layout,
            page_sampler,
            page_bind_group,
            pipeline,
        })
    }

    pub(super) fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.page_texture =
            create_page_texture(&self.device, &self.queue, &self.config, &mut self.page);
        let view = self
            .page_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.page_bind_group =
            create_page_bind_group(&self.device, &self.page_layout, &self.page_sampler, &view);
    }

    pub(super) fn render(&mut self, window: &NativeWindow) -> Result<(), WindowError> {
        self.page.redraw(&self.queue, &self.page_texture);
        let (frame, suboptimal) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(WindowError::SurfaceValidation);
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ctml-window-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ctml-window-clear-pass"),
                color_attachments: &color_attachments,
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.page_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        self.queue.present(frame);

        if suboptimal {
            self.surface.configure(&self.device, &self.config);
        }
        Ok(())
    }

    pub(super) fn advance_animations(&mut self, now: Instant) -> Option<bool> {
        let active = self.page.advance_animations(now)?;
        self.page.redraw(&self.queue, &self.page_texture);
        Some(active)
    }

    pub(super) fn pointer_moved(
        &mut self,
        position: PhysicalPosition<f64>,
    ) -> (CursorIcon, bool, bool) {
        let hover_changed = self.page.update_hover(position.x, position.y);
        let selection_changed = self.page.extend_selection(position.x, position.y);
        let (cursor, visible) = self.page.cursor_at(position.x, position.y);
        (cursor, visible, hover_changed || selection_changed)
    }

    pub(super) fn scroll_by(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        pointer_x: f64,
        pointer_y: f64,
    ) -> bool {
        if !self
            .page
            .scroll_by_at(delta_x, delta_y, pointer_x, pointer_y)
        {
            return false;
        }
        self.page.redraw(&self.queue, &self.page_texture);
        true
    }

    pub(super) fn pointer_left(&mut self) -> bool {
        self.page.clear_hover()
    }

    pub(super) fn tab_focus(&mut self, backwards: bool) -> bool {
        self.page.focus_next(backwards)
    }

    pub(super) fn select_word_at(&mut self, position: PhysicalPosition<f64>) -> bool {
        self.page.select_word_at(position.x, position.y)
    }

    pub(super) fn mouse_input(
        &mut self,
        state: ElementState,
        button: MouseButton,
        position: PhysicalPosition<f64>,
    ) -> bool {
        if button != MouseButton::Left {
            return false;
        }
        match state {
            ElementState::Pressed => self.page.begin_selection(position.x, position.y),
            ElementState::Released => {
                self.page.end_selection();
                self.page.end_active()
            }
        }
    }
}

fn create_page_renderer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
    page: &mut PageRasterizer,
) -> Result<
    (
        wgpu::Texture,
        wgpu::BindGroupLayout,
        wgpu::Sampler,
        wgpu::BindGroup,
        wgpu::RenderPipeline,
    ),
    WindowError,
> {
    let texture = create_page_texture(device, queue, config, page);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ctml-page-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let bind_group = create_page_bind_group(device, &layout, &sampler, &view);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ctml-page-shader"),
        source: wgpu::ShaderSource::Wgsl(r#"
            @group(0) @binding(0) var page_texture: texture_2d<f32>;
            @group(0) @binding(1) var page_sampler: sampler;
            struct VertexOutput { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> };
            @vertex fn vertex_main(@builtin(vertex_index) index: u32) -> VertexOutput {
                var positions = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
                var uvs = array<vec2<f32>, 3>(vec2(0.0, 1.0), vec2(2.0, 1.0), vec2(0.0, -1.0));
                var output: VertexOutput;
                output.position = vec4(positions[index], 0.0, 1.0);
                output.uv = uvs[index];
                return output;
            }
            @fragment fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
                return textureSample(page_texture, page_sampler, input.uv);
            }
        "#.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ctml-page-pipeline-layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ctml-page-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    Ok((texture, layout, sampler, bind_group, pipeline))
}

fn create_page_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
    page: &mut PageRasterizer,
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ctml-page-texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    page.resize(queue, &texture, config.width, config.height);
    texture
}

fn create_page_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ctml-page-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}
