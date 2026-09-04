mod gltf;

use std::path::Path;

use ::gltf::animation::{Interpolation, Property};
pub use gltf::load;
use image::DynamicImage;

use crate::{
    game::ModelId,
    maths::{Quat, Vec3},
};

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

#[derive(Debug, Clone)]
pub struct AnimationChannel {
    pub property: Property,
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    pub values: AnimationValues,
}

#[derive(Debug, Clone)]
pub enum AnimationValues {
    Translation(Vec<[f32; 3]>),
    Rotation(Vec<[f32; 4]>),
    Scale(Vec<[f32; 3]>),
}

fn keyframes(times: &[f32], time: f32) -> (usize, usize, f32) {
    if times.len() <= 1 {
        return (0, 0, 0.0);
    }

    if time <= times[0] {
        return (0, 0, 0.0);
    }

    let last = times.len() - 1;

    if time >= times[last] {
        return (last, last, 0.0);
    }

    for i in 0..last {
        if time >= times[i] && time < times[i + 1] {
            let duration = times[i + 1] - times[i];
            let t = (time - times[i]) / duration;

            return (i, i + 1, t);
        }
    }

    (last, last, 0.0)
}
fn sample_quat(times: &[f32], values: &[[f32; 4]], time: f32) -> Quat {
    let (i0, i1, t) = keyframes(times, time);

    let a = Quat::from(values[i0]);
    let b = Quat::from(values[i1]);

    a.slerp(b, t)
}

fn sample_vec3(times: &[f32], values: &[[f32; 3]], time: f32) -> Vec3 {
    let (i0, i1, t) = keyframes(times, time);

    let a = Vec3::from(values[i0]);
    let b = Vec3::from(values[i1]);

    a + (b - a) * t
}
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub channels: Vec<AnimationChannel>,
    pub duration: f32,
}

impl AnimationClip {
    pub fn sample(&self, time: f32) -> (Vec3, Quat, Vec3) {
        let mut translation = Vec3::zeroes();
        let mut rotation = Quat::identity();
        let mut scale = Vec3::xyz(1.0);

        for channel in &self.channels {
            match &channel.values {
                AnimationValues::Translation(values) => {
                    translation = sample_vec3(&channel.times, values, time);
                }

                AnimationValues::Rotation(values) => {
                    rotation = sample_quat(&channel.times, values, time);
                }

                AnimationValues::Scale(values) => {
                    scale = sample_vec3(&channel.times, values, time);
                }
            }
        }

        (translation, rotation, scale)
    }
}
