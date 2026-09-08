use std::sync::Arc;

use gpu::Gpu;
use winit::{dpi::PhysicalSize, window::Window};

use crate::assets::AssetModels;
use crate::game::Game;
use crate::graphics::gpu::GpuModels;

mod gpu;
pub use gpu::Vertex;

pub struct State {
    pub window: Arc<Window>,
    pub gpu: Gpu,
    pub models: GpuModels,
}

impl State {
    pub fn new(window: Window, assets: &AssetModels) -> Self {
        let window = Arc::new(window);
        let window_size = window.inner_size();

        let gpu = Gpu::new(window.clone(), window_size.width, window_size.height);

        // TODO: Avoid this clone
        let models = GpuModels::load(&gpu, assets.0.clone());

        Self {
            window,
            gpu,
            models,
        }
    }

    #[inline(always)]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size.width, size.height);
    }

    #[inline(always)]
    pub fn render(&mut self, game: &Game, assets: &AssetModels) {
        self.gpu.render(&self.window, game, &self.models, assets);
    }
}
