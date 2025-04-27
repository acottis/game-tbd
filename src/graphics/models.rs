use wgpu::{
    VertexAttribute, VertexBufferLayout, VertexStepMode, vertex_attr_array,
};

use crate::math::{Mat4, Vec3};

use super::assets::Material;

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex3D {
    vec3: Vec3,
    uv: [f32; 2],
}
impl Vertex3D {
    const ATTRIBUTES: [VertexAttribute; 2] =
        vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn new(vec3: Vec3, uv: [f32; 2]) -> Self {
        Self { vec3, uv }
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
pub struct Vertex3DUniform {
    translation: Mat4,
}

impl Vertex3DUniform {
    fn new(translation: Mat4) -> Self {
        Self { translation }
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
