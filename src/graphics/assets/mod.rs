mod gltf;

use image::DynamicImage;
use wgpu::{
    VertexAttribute, VertexBufferLayout, VertexStepMode, vertex_attr_array,
};

use crate::math::Vec3;
pub use gltf::load_glb;

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    vec3: Vec3,
    normal: Vec3,
    uv: [f32; 2],
}
impl Vertex {
    const ATTRIBUTES: [VertexAttribute; 3] =
        vertex_attr_array![0 => Float32x3, 1 => Float32x3 ,2 => Float32x2];

    pub fn new(vec3: Vec3, normal: Vec3, uv: [f32; 2]) -> Self {
        Self { vec3, normal, uv }
    }

    pub const fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
pub struct MaterialUniform {
    base_colour: [f32; 4],
    metallic: f32,
    roughness: f32,
    has_texture: u32,
    _padding: [u8; 4],
}

impl From<&Material> for MaterialUniform {
    fn from(value: &Material) -> Self {
        Self {
            base_colour: value.base_colour,
            metallic: value.metallic,
            roughness: value.roughness,
            has_texture: value.image.is_some() as _,
            _padding: Default::default(),
        }
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

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
}
impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        material: Material,
    ) -> Self {
        Self {
            vertices,
            indices,
            material,
        }
    }
}

