
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::{self, ActiveEventLoop, EventLoop}, window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Window>,
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

