use std::sync::Arc;

use gpu::Gpu;
use winit::{dpi::PhysicalSize, window::Window};

use crate::assets::{AssetModel, AssetModels};
use crate::game::Game;
use crate::graphics::gpu::Models;

mod gpu;
pub use gpu::Vertex;

pub struct State {
    pub window: Arc<Window>,
    pub gpu: Gpu,
    pub models: Models,
}

impl State {
    pub fn new(window: Window, assets: &[AssetModel]) -> Self {
        let window = Arc::new(window);
        let gpu = Gpu::new(window.clone());

        let models = Models::load(&gpu, &assets);

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
    pub fn render(&mut self, game: &Game, assets: &AssetModels) {
        self.gpu.render(&self.window, game, &self.models, assets);
    }
}
