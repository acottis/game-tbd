use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::assets::{AssetModel, AssetModelSet};
use crate::game::Game;
use crate::graphics::{Gpu, ModelSet};

pub struct Handle {
    generation: u32,
    index: usize,
}

pub struct Engine {
    pub window: Arc<Window>,
    pub gpu: Gpu,
    pub models: ModelSet,
}

impl Engine {
    pub fn new(window: Window, assets: &[AssetModel]) -> Self {
        let window = Arc::new(window);
        let mut gpu = Gpu::new(window.clone());
        gpu.resize(window.inner_size());

        let models = ModelSet::load(&gpu, &assets);

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
    pub fn render(&mut self, game: &Game, assets: &AssetModelSet) {
        self.gpu.render(&self.window, game, &self.models, assets);
    }
}
