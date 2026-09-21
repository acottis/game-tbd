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

    pub fn shadow_transform(&self, target: Vec3) -> Mat4 {
        const ORTHO_EXTENT: f32 = 150.0;
        const CAMERA_DISTANCE: f32 = 250.0;
        const NEAR: f32 = 0.1;
        const FAR: f32 = 500.0;
        let direction = self.direction.normalize();
        let position = target - direction * CAMERA_DISTANCE;

        let proj = proj::directx::orthographic(
            -ORTHO_EXTENT,
            ORTHO_EXTENT,
            -ORTHO_EXTENT,
            ORTHO_EXTENT,
            NEAR,
            FAR,
        );
        let view = view::look_at_mat4(position, target, Vec3::Y);

        proj * view
    }
}
