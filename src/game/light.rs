use bytemuck::{Pod, Zeroable};
use glam::{
    Mat4, Vec3,
    camera::rh::{proj, view},
};

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

    // TODO: Fix this, shadows only work in certain area
    pub fn shadow_transform(&self, target: Vec3) -> Mat4 {
        let direction = self.direction.normalize();
        let position = target - direction * 50.0;

        let proj = proj::directx::orthographic(-40.0, 40.0, -40.0, 20.0, 0.1, 100.0);
        let view = view::look_at_mat4(position, target, Vec3::Y);

        proj * view
    }
}
