use std::{f32::consts::PI, time::Instant};

use game::Game;
use input::Input;
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
mod input;
mod math;
mod physics;
use graphics::State;

struct App {
    state: Option<State>,
    game: Game,
    input: Input,
    last_frame_time: Instant,
    delta_time: f32,
}

impl App {
    fn new() -> Self {
        Self {
            state: None,
            game: Game::new(),
            input: Input::new(),
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

    fn run_input(&mut self, event_loop: &ActiveEventLoop) {
        let player = &mut self.game.entities[1];
        let camera = &mut self.state.as_mut().unwrap().camera;
        if self.input.is_pressed(KeyCode::KeyW) {
            player.move_x(self.delta_time, 10.0);
        }
        if self.input.is_pressed(KeyCode::KeyA) {
            player.move_z(self.delta_time, -10.0);
        }
        if self.input.is_pressed(KeyCode::KeyS) {
            player.move_x(self.delta_time, -10.0);
        }
        if self.input.is_pressed(KeyCode::KeyD) {
            player.move_z(self.delta_time, 10.0);
        }
        if self.input.is_pressed(KeyCode::Space) {
            player.move_y(self.delta_time, 10.0);
        }
        if self.input.is_pressed(KeyCode::ArrowUp) {
            camera.forward(self.delta_time, 10.0)
        }
        if self.input.is_pressed(KeyCode::ArrowLeft) {
            camera.strafe(self.delta_time, -1.0);
        }
        if self.input.is_pressed(KeyCode::ArrowDown) {
            camera.forward(self.delta_time, -10.0)
        }
        if self.input.is_pressed(KeyCode::ArrowRight) {
            camera.strafe(self.delta_time, 1.0);
        }
        if self.input.is_pressed(KeyCode::KeyU) {
            camera.rotate_z(self.delta_time, PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyH) {
            camera.rotate_y(self.delta_time, -PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyJ) {
            camera.rotate_z(self.delta_time, -PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyK) {
            camera.rotate_y(self.delta_time, PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::Escape) {
            event_loop.exit();
        }
    }

    fn run_game(&mut self) {
        self.game.update(self.delta_time);
    }

    fn update_delta_time(&mut self) {
        let now = Instant::now();
        self.delta_time =
            now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        //println!("FPS: {}", 1.0 / self.delta_time);
        //println!("FPS: {}", self.delta_time);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes =
            Window::default_attributes().with_title("WIP: Game");
        let window = event_loop.create_window(window_attributes).unwrap();
        self.init(window);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.update_delta_time();
        self.run_input(event_loop);
        self.run_game();
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
                self.input.handle_keyboard(event);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, direction) => {
                    if direction == -1.0 {
                        self.state
                            .as_mut()
                            .unwrap()
                            .camera
                            .forward(self.delta_time, -100.0);
                    }
                    if direction == 1.0 {
                        self.state
                            .as_mut()
                            .unwrap()
                            .camera
                            .forward(self.delta_time, 100.0);
                    }
                }
                MouseScrollDelta::PixelDelta(_) => (),
            },
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
