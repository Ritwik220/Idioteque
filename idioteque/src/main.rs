
use std::sync::Arc;
use std::default::Default;
use wgpu::{Operations, TextureViewDescriptor};
// use wgpu::naga::SwitchValue::Default;
use winit::{
    application::ApplicationHandler, event::{KeyEvent, WindowEvent}, event_loop::{self, ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId},
};



// State for wgpu
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_config: bool,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline
}


#[derive(Default)]
struct App {
    state: Option<State>,
    window: Option<Arc<Window>>
}

impl State {
    // Creating the state
    async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        }).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // the configuration of the state, letss gooo
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };


        // Lodaing the shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wsgl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("REnder pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default()
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview_mask: None,
            cache: None
        });

        Ok(Self {
            surface,
            device,
            config,
            queue,
            is_surface_config: false,
            window,
            render_pipeline
        })
    }


    // Resize
    fn resize(&mut self, widht:u32, height:u32) {
        if(widht > 0 && height > 0) {
            self.config.height = height;
            self.config.width = widht;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_config = true;
        }
    }

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match(code, is_pressed) {
            (KeyCode::Escape, true) => {event_loop.exit();}
            _ => {}
        };
    }

    fn update(&mut self) {}

    fn render(&mut self, r: f64, b: f64, g: f64) -> anyhow::Result<()> {
        self.window.request_redraw();
        if !self.is_surface_config {
            return Ok(());
        }

        // Creating the surface texture
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // Skip this frame
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // You could recreate the devices and all resources
                // created with it here, but we'll just bail
                anyhow::bail!("Lost device");
            }

        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Created encoder to be able to give commands to the gpu
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice:None,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r,
                            g: g,
                            b: b,
                            a: 1.0
                        }),
                        store: wgpu::StoreOp::Store
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
            
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())

    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Creating window...");
        let attributes = Window ::default_attributes()
            .with_title("Idioteque")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

        let window = Arc::new(event_loop
            .create_window(attributes)
            .expect("Failed to create window"));

        window.set_visible(true);
        println!("Window created: {:?}", window.id());
        window.set_visible(true);
        window.focus_window();
        self.state = Some(pollster::block_on(State::new(window.clone())).unwrap());
        self.window = Some(window);
    }

    // Handling window events
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
                println!("Window resized: {} x {}", size.width, size.height);
            }

            WindowEvent::CursorMoved { device_id, position } => {
                let _ = state.render(0.5, 0.6, 0.7);
            }

            WindowEvent::RedrawRequested => {
                println!("Redraw requested");
                state.update();
                match state.render(1.0 , 0.2, 0.3) {
                    Ok(_) => {},
                    Err(err) => {
                        println!("Err: {}", err);
                    }
                }
            }

            

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),

            _ => {}
        }
    }
}

fn main(){
    let event_loop = EventLoop::new().expect("Failed to create an event loop");

    let mut app = App::default();

    event_loop
        .run_app(&mut app)
        .expect("Event loop failed");

}

