use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

use game::Game;
use glam::Vec3;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::{Window, WindowId},
};

mod game;
mod graphics;
use graphics::State;

use crate::{
    game::{Entity, ModelId, input::Input},
    graphics::Transform,
};

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
        let state = State::new(window);

        let ground = Entity::new(
            Vec3::ZERO,
            Vec3::splat(80.0),
            false,
            ModelId::Ground,
            Transform::new(&state.gpu),
        );
        let cube = Entity::new(
            Vec3::ZERO,
            Vec3::splat(0.3),
            true,
            ModelId::Foo,
            Transform::new(&state.gpu),
        );
        self.game.entities.extend([ground, cube]);

        self.state = Some(state)
    }

    #[inline(always)]
    fn render(&mut self) {
        let state = unsafe { self.state.as_mut().unwrap_unchecked() };
        state.render(&self.game.entities);
    }

    #[inline(always)]
    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width * size.height != 0 {
            let state = unsafe { self.state.as_mut().unwrap_unchecked() };
            state.resize(size);
        }
    }

    fn handle_inputs(&mut self, event_loop: &ActiveEventLoop) {
        let state = unsafe { self.state.as_mut().unwrap_unchecked() };
        let player = &mut self.game.entities[1];
        let camera = &mut state.camera;

        // Movement is relative to camera direction
        let mut movement = Vec3::ZERO;
        if self.input.is_pressed(KeyCode::KeyW) {
            movement += camera.forward_planar()
        }
        if self.input.is_pressed(KeyCode::KeyS) {
            movement -= camera.forward_planar()
        }
        if self.input.is_pressed(KeyCode::KeyA) {
            movement -= camera.right()
        }
        if self.input.is_pressed(KeyCode::KeyD) {
            movement += camera.right()
        }
        if self.input.is_pressed(KeyCode::Space) {
            player.jump(self.delta_time * 100.0);
        }
        if self.input.is_pressed(KeyCode::ArrowUp) {
            camera.move_forward(self.delta_time * 10.0)
        }
        if self.input.is_pressed(KeyCode::ArrowLeft) {
            camera.strafe(self.delta_time * -10.0);
        }
        if self.input.is_pressed(KeyCode::ArrowDown) {
            camera.move_forward(self.delta_time * -10.0)
        }
        if self.input.is_pressed(KeyCode::ArrowRight) {
            camera.strafe(self.delta_time * 10.0);
        }
        if self.input.is_pressed(KeyCode::KeyU) {
            camera.rotate_pitch(self.delta_time * PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyJ) {
            camera.rotate_pitch(self.delta_time * -PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyH) {
            camera.rotate_yaw(self.delta_time * -PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyK) {
            camera.rotate_yaw(self.delta_time * PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::Escape) {
            event_loop.exit();
        }
        player.move_direction(self.delta_time * 5.0, movement);
        camera.follow(player.position());
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("WIP: Game");
        let window = event_loop.create_window(window_attributes).unwrap();
        self.init(window);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if now.duration_since(self.last_frame_time) <= Duration::from_millis(1000 / 24) {
            return;
        }
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        log::debug!("FPS: {}, DT: {}", 1.0 / self.delta_time, self.delta_time);
        self.last_frame_time = now;

        self.handle_inputs(event_loop);
        self.game.update(self.delta_time);
        self.render();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
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
                    let state = unsafe { self.state.as_mut().unwrap_unchecked() };
                    state.camera.zoom(direction);
                }
                MouseScrollDelta::PixelDelta(_) => (),
            },
            // Ignored events
            WindowEvent::Moved(_) => {}
            WindowEvent::CursorMoved { .. } => {}
            _ => log::info!("{event:?}"),
        };
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::new()).unwrap();
}
