use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod graphics;
mod math;
use graphics::State;

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl App {
    fn init(&mut self, window: Window) {
        self.state = Some(State::new(window))
    }

    fn render(&mut self) {
        self.state.as_mut().unwrap().render();
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width * size.height == 0 {
            return;
        }
        self.state.as_mut().unwrap().resize(size);
    }
    fn handle_input(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
            PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.state.as_mut().unwrap().camera.strafe(-0.01);
                self.render();
            }
            PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.state.as_mut().unwrap().camera.strafe(0.01);
                self.render();
            }
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.state.as_mut().unwrap().camera.forward(0.01);
                self.render();
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.state.as_mut().unwrap().camera.forward(-0.01);
                self.render();
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes =
            Window::default_attributes().with_title("WIP: Game");
        let window = event_loop.create_window(window_attributes).unwrap();
        self.init(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                self.render();
            }
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { ref event, .. } => {
                self.handle_input(event_loop, event)
            }
            // Ignored events
            WindowEvent::Moved(_) => {}
            WindowEvent::CursorMoved { .. } => {}
            _ => println!("{event:?}"),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default()).unwrap();
}
