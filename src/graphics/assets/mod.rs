mod gltf;

use ::gltf::animation::{Interpolation, Property};
pub use gltf::load;
use image::DynamicImage;

use super::{Vertex, gpu::MaterialUniform};

impl From<&Material> for MaterialUniform {
    fn from(m: &Material) -> Self {
        MaterialUniform::new(m.base_colour, m.metallic, m.roughness, m.image.is_some())
    }
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
}

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

#[derive(Debug)]
pub struct AnimationClip {
    pub channels: Vec<AnimationChannel>,
    pub duration: f32,
}

#[derive(Debug)]
pub struct AnimationChannel {
    pub property: Property,
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    pub values: AnimationValues,
}

#[derive(Debug)]
pub enum AnimationValues {
    Translation(Vec<[f32; 3]>),
    Rotation(Vec<[f32; 4]>),
    Scale(Vec<[f32; 3]>),
}
