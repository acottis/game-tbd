mod gltf;

use std::path::Path;

pub use gltf::load;
use image::DynamicImage;

use crate::{animation::AnimationClip, game::ModelId};

use super::{Vertex, gpu::MaterialUniform};

impl From<&Material> for MaterialUniform {
    fn from(m: &Material) -> Self {
        MaterialUniform::new(m.base_colour, m.metallic, m.roughness, m.image.is_some())
    }
}

#[derive(Clone)]
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
pub struct AssetModels(pub [AssetModel; 3]);

impl AssetModels {
    pub fn load(paths: [impl AsRef<Path>; 3]) -> Self {
        Self(paths.map(|path| load(path)))
    }

    pub fn get(&self, id: ModelId) -> &AssetModel {
        &self.0[id as usize]
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, material: Material) -> Self {
        Self {
            vertices,
            indices,
            material,
        }
    }
}
