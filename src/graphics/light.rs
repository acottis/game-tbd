use bytemuck::{Pod, Zeroable};

use crate::maths::Vec3;

#[derive(Zeroable, Pod, Copy, Clone)]
#[repr(C)]
pub struct Light {
    position: Vec3,
    _padding: [u8; 4],
    color: Vec3,
    intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
            _padding: Default::default(),
        }
    }
}
