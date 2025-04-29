use std::sync::Arc;

use assets::load_glb;

use gpu::Gpu;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{game::Entity, math::Vec3};

mod assets;
mod camera;
mod gpu;
mod light;
pub use camera::Camera;
pub use gpu::MeshInstance;
pub use gpu::Vertex;
pub use light::Light;

pub struct Graphics {
    pub window: Arc<Window>,
    pub camera: Camera,
    pub gpu: Gpu,
}

impl Graphics {
    pub fn new(window: Window) -> Self {
        let window = Arc::new(window);
        let window_size = window.inner_size();

        let camera = Camera::new(&window_size);
        let light = Light::new(
            Vec3::new(0.0, 0.5, 0.5),
            Vec3::new(1.0, 1.0, 0.0),
            0.75,
        );

        let gpu = Gpu::new(
            window.clone(),
            window_size.width,
            window_size.height,
            &camera,
            &light,
        );

        Self {
            window,
            camera,
            gpu,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size.width, size.height);
        self.camera.set_aspect_ratio(&size);
    }

    pub fn render(&mut self, entities: &Vec<Entity>) {
        self.gpu.write_camera(&self.camera.view_perspective_rh());

        let frame = self.gpu.render(entities);
        self.window.pre_present_notify();
        frame.present();
    }
}

pub fn load_assets() -> impl Iterator<Item = assets::Mesh> {
    [
        load_glb("assets/BoxTextured.glb"),
        load_glb("assets/cube.glb"),
        load_glb("assets/ground.glb"),
    ]
    .into_iter()
    .flatten()
}

pub enum MeshInstanceId {
    Ground,
    Cube,
    CubeGltf,
}
