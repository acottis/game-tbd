mod gltf;

use ::gltf::mesh::BoundingBox;
pub use gltf::load;
use image::DynamicImage;

use crate::{game::animation::AnimationClip, graphics::Vertex};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ModelId {
    Foo = 0,
    _Cube = 1,
    Ground = 2,
    Foo2 = 3,
}

#[derive(Clone, Debug)]
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

#[derive(Clone)]
pub struct AssetModel {
    pub meshes: Vec<Mesh>,
    pub animations: Vec<AnimationClip>,
}
pub struct AssetModels(pub Vec<AssetModel>);

impl AssetModels {
    pub fn load() -> Self {
        let paths = [
            "assets/foo.glb",
            "assets/cube.glb",
            "assets/ground.glb",
            "assets/foo2.glb",
        ];
        Self(paths.into_iter().map(|path| load(path)).collect())
    }

    pub fn get(&self, id: ModelId) -> &AssetModel {
        &self.0[id as usize]
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
    pub bounding_box: BoundingBox,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        material: Material,
        bounding_box: BoundingBox,
    ) -> Self {
        Self {
            vertices,
            indices,
            material,
            bounding_box,
        }
    }
}
