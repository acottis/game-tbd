use bytemuck::{Pod, Zeroable};
use glam::Vec3;

#[derive(Zeroable, Pod, Copy, Clone)]
#[repr(C)]
pub struct Light {
    direction: Vec3,
    intensity: f32,
    color: Vec3,
    ambient: f32,
}

impl Light {
    pub const SIZE: usize = size_of::<Self>();

    pub fn new(direction: Vec3, color: Vec3, intensity: f32, ambient: f32) -> Self {
        Light {
            direction,
            color,
            intensity,
            ambient,
        }
    }
}
