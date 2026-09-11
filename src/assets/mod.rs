mod gltf;

use ::gltf::mesh::BoundingBox;
use glam::Mat4;
pub use gltf::load;
use image::DynamicImage;

use crate::{game::animation::AnimationClip, graphics::Vertex};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ModelId {
    Foo = 0,
    _Cube = 1,
    Ground = 2,
}

pub struct Material {
    pub base_colour: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub image: Option<DynamicImage>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_colour: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 1.0,
            image: None,
        }
    }
}

pub struct AssetModel {
    pub meshes: Vec<Mesh>,
    pub animations: Vec<AnimationClip>,
    pub materials: Vec<Material>,
}
pub struct AssetModels(pub Vec<AssetModel>);

impl AssetModels {
    pub fn load() -> Self {
        let paths = ["assets/foo.glb", "assets/cube.glb", "assets/ground.glb"];
        Self(paths.into_iter().map(|path| load(path)).collect())
    }

    pub fn get(&self, id: ModelId) -> &AssetModel {
        &self.0[id as usize]
    }
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
    pub transform: Mat4,
}

#[derive(Clone, Debug)]
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounding_box: BoundingBox,
    pub material: Option<usize>,
}

impl Primitive {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        bounding_box: BoundingBox,
        material: Option<usize>,
    ) -> Self {
        Self {
            vertices,
            indices,
            bounding_box,
            material,
        }
    }
}
