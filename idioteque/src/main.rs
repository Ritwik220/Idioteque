
use std::sync::Arc;
use std::default::Default;
// use wgpu::naga::SwitchValue::Default;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::{self, ActiveEventLoop, EventLoop}, window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Window>,
}


// State for wgpu
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_config: bool,
    window: Arc<Window>
}


impl State {
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



        Ok(Self {
            surface,
            device,
            config,
            queue,
            is_surface_config: false,
            window
        })
    }
}


impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Creating window...");
        let attributes = Window ::default_attributes()
            .with_title("Idoteque")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));

        let window = event_loop
            .create_window(attributes)
            .expect("Failed to create window");

        window.set_visible(true);
        println!("Window created: {:?}", window.id());
        window.set_visible(true);
        window.focus_window();
        self.window = Some(window);
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                println!("Window resized: {} x {}", size.width, size.height);
            }

            WindowEvent::RedrawRequested => {
                println!("Redraw requested");
            }

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

