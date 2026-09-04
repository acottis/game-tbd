use std::sync::Arc;

use assets::load;

use gpu::Gpu;
use winit::{dpi::PhysicalSize, window::Window};

use crate::graphics::assets::AssetModels;
use crate::graphics::gpu::GpuModels;
use crate::{game::Entity, maths::Vec3};

mod assets;
mod camera;
mod gpu;
mod light;
pub use camera::Camera;
pub use gpu::ModelTransform;
pub use gpu::Vertex;
pub use light::Light;

pub struct State {
    pub window: Arc<Window>,
    pub camera: Camera,
    pub gpu: Gpu,
    pub models: GpuModels,
    pub asset_models: AssetModels,
}

impl State {
    pub fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let window_size = window.inner_size();

        let camera = Camera::new(&window_size);
        let light = Light::new(Vec3::new(0.0, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0), 0.9);

        let gpu = Gpu::new(
            window.clone(),
            window_size.width,
            window_size.height,
            &camera,
            &light,
        );

        let asset_models =
            AssetModels::load(["assets/foo.glb", "assets/cube.glb", "assets/ground.glb"]);
        // TODO: Avoid this clone
        let models = GpuModels::load(&gpu, asset_models.0.clone());

        Self {
            window,
            camera,
            gpu,
            models,
            asset_models,
        }
    }

    #[inline(always)]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size.width, size.height);
        self.camera.set_aspect_ratio(&size);
    }

    #[inline(always)]
    pub fn render(&mut self, entities: &[Entity]) {
        self.gpu.render(
            &self.window,
            entities,
            &self.camera,
            &self.models,
            &self.asset_models,
        );
    }
}
