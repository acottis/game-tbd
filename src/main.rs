use std::{f32::consts::PI, time::Instant};

use game::Game;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod game;
mod graphics;
mod math;
mod physics;
use graphics::State;

struct App {
    state: Option<State>,
    game: Game,
    last_frame_time: Instant,
    delta_time: f32,
}

impl App {
    fn new() -> Self {
        Self {
            state: None,
            game: Game::new(),
            last_frame_time: Instant::now(),
            delta_time: 0.0,
        }
    }

    fn init(&mut self, window: Window) {
        let mut state = State::new(window);

        let meshes = graphics::load_assets();
        state.gpu.load_meshes(meshes);

        self.game.init(&state);
        self.state = Some(state)
    }

    #[inline(always)]
    fn state(&mut self) -> &mut State {
        unsafe { self.state.as_mut().unwrap_unchecked() }
    }

    #[inline(always)]
    fn render(&mut self) {
        self.state.as_mut().unwrap().render(&self.game.entities);
    }

    #[inline(always)]
    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width * size.height == 0 {
            return;
        }
        self.state().resize(size);
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        let dt = self.delta_time;
        println!("{event:?}");
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
            PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.state().camera.strafe(dt, -10.0)
            }
            PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.state().camera.strafe(dt, 10.0)
            }
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.state().camera.forward(dt, 100.0)
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.state().camera.forward(dt, -100.0)
            }
            PhysicalKey::Code(KeyCode::KeyH) => {
                self.state().camera.rotate_y(PI / 16.0)
            }
            PhysicalKey::Code(KeyCode::KeyK) => {
                self.state().camera.rotate_y(-PI / 16.0)
            }
            PhysicalKey::Code(KeyCode::KeyU) => {
                self.state().camera.rotate_x(PI / 16.0)
            }
            PhysicalKey::Code(KeyCode::KeyJ) => {
                self.state().camera.rotate_x(-PI / 16.0)
            }
            PhysicalKey::Code(KeyCode::KeyW) => {
                let player = &mut self.game.entities[1];
                player.move_x(dt, 100.0);
            }
            PhysicalKey::Code(KeyCode::KeyA) => {
                let player = &mut self.game.entities[1];
                player.move_z(dt, -100.0);
            }
            PhysicalKey::Code(KeyCode::KeyS) => {
                let player = &mut self.game.entities[1];
                player.move_x(dt, -100.0);
            }
            PhysicalKey::Code(KeyCode::KeyD) => {
                let player = &mut self.game.entities[1];
                player.move_z(dt, 100.0);
            }
            PhysicalKey::Code(KeyCode::Space) => {
                let player = &mut self.game.entities[1];
                player.jump(dt, 1000.0);
            }
            _ => {}
        }
    }

    fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        let dt = self.delta_time;
        match delta {
            MouseScrollDelta::LineDelta(_, direction) => {
                if direction == -1.0 {
                    self.state().camera.forward(dt, -100.0);
                }
                if direction == 1.0 {
                    self.state().camera.forward(dt, 100.0);
                }
            }
            MouseScrollDelta::PixelDelta(_) => (),
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

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let now = Instant::now();
        self.delta_time =
            now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        //println!("FPS: {}", 1.0 / self.delta_time);
        //println!("FPS: {}", self.delta_time);

        self.game.update(self.delta_time);

        self.render();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { ref event, .. } => {
                self.handle_key(event_loop, event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_scroll(delta);
            }
            // Ignored events
            WindowEvent::Moved(_) => {}
            WindowEvent::CursorMoved { .. } => {}
            _ => println!("{event:?}"),
        };
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::new()).unwrap();
}
