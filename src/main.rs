use std::{sync::Arc, time::Instant};

use game::Game;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

mod assets;
mod engine;
mod game;
mod graphics;
mod input;

use crate::assets::{Asset, AssetSet};

pub struct Renderer {
    pub window: Arc<Window>,
    pub gpu: graphics::Gpu,
    pub models: graphics::ModelSet,
}

impl Renderer {
    pub fn new(window: Window, assets: &[Asset]) -> Self {
        let window = Arc::new(window);
        let mut gpu = graphics::Gpu::new(window.clone());
        gpu.resize(window.inner_size());

        let models = graphics::ModelSet::load(&gpu, &assets);

        Self {
            window,
            gpu,
            models,
        }
    }

    #[inline(always)]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size);
    }

    #[inline(always)]
    pub fn render(&mut self, game: &Game, assets: &AssetSet) {
        self.gpu.render(&self.window, game, &self.models, assets);
    }
}

struct App {
    renderer: Option<Renderer>,
    game: Game,
    assets: AssetSet,
    last_frame_time: Instant,
    delta_time: f32,
}

impl App {
    fn new() -> Self {
        let assets = AssetSet::load();
        Self {
            renderer: None,
            game: Game::new(&assets),
            assets,
            last_frame_time: Instant::now(),
            delta_time: 0.0,
        }
    }

    fn init(&mut self, window: Window) {
        self.renderer = Some(Renderer::new(window, &self.assets.0));
    }

    #[inline(always)]
    fn render(&mut self) {
        let renderer = unsafe { self.renderer.as_mut().unwrap_unchecked() };
        renderer.render(&self.game, &self.assets);
    }

    #[inline(always)]
    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        let renderer = unsafe { self.renderer.as_mut().unwrap_unchecked() };
        renderer.resize(size);
        self.game.camera.set_aspect_ratio(&size);
    }

    fn handle_inputs(&mut self, event_loop: &ActiveEventLoop) {
        let should_exit = self.game.handle_inputs(self.delta_time);
        if should_exit {
            event_loop.exit()
        }
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
        let delta = now.duration_since(self.last_frame_time);
        // const FPS: u64 = 60;
        // if delta <= std::time::Duration::from_millis(1000 / FPS) {
        //     return;
        // }
        self.last_frame_time = now;
        self.delta_time = delta.as_secs_f32();

        log::debug!("FPS: {}, DT: {}", 1.0 / self.delta_time, self.delta_time);

        self.handle_inputs(event_loop);
        self.game.update(self.delta_time, &self.assets);
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
                self.game.input.handle_keyboard(event);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, direction) => {
                    self.game.camera.zoom(direction);
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
