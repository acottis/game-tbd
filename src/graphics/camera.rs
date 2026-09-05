use std::f32::consts::PI;

use glam::camera::rh;
use glam::{Mat4, Quat, Vec3};
use winit::dpi::PhysicalSize;

const MAX_ZOOM: f32 = 50.0;
const MIN_ZOOM: f32 = 1.5;

#[derive(Debug)]
pub struct Camera {
    /// Our position (eye)
    position: Vec3,
    /// The center of what we are looking at, rotations are relative to target
    target: Vec3,
    up: Vec3,
    /// Field of view
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl Camera {
    pub const fn new(window_size: &PhysicalSize<u32>) -> Self {
        Self {
            position: Vec3::new(-2.0, 2.0, 0.0),
            target: Vec3::new(0.0, 1.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fovy: PI / 4.0,
            aspect: window_size.width as f32 / window_size.height as f32,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn set_aspect_ratio(&mut self, size: &PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32
    }

    pub fn follow(&mut self, target: Vec3) {
        let offset = self.position - self.target;

        self.target = target;
        self.position = target + offset;
    }

    pub fn forward(&self) -> Vec3 {
        (self.target - self.position).normalize_or_zero()
    }

    pub fn forward_planar(&self) -> Vec3 {
        let mut forward = self.target - self.position;
        forward.y = 0.0;
        forward.normalize_or_zero()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up).normalize_or_zero()
    }

    /// Orbit camera around target horizontally
    pub fn rotate_yaw(&mut self, angle: f32) {
        let rotation = Quat::from_axis_angle(Vec3::Y, angle);
        let offset = self.position - self.target;
        self.position = self.target + rotation * offset;
    }

    /// Orbit camera around target vertically
    pub fn rotate_pitch(&mut self, angle: f32) {
        let right = self.right();
        let rotation = Quat::from_axis_angle(right, angle);
        let offset = self.position - self.target;
        self.position = self.target + rotation * offset;
    }

    pub fn move_forward(&mut self, distance: f32) {
        let delta = self.forward_planar() * distance;
        self.position += delta;
        self.target += delta;
    }

    pub fn strafe(&mut self, distance: f32) {
        let delta = self.right() * distance;
        self.position += delta;
        self.target += delta;
    }

    pub fn zoom(&mut self, amount: f32) {
        let delta = self.position - self.target;
        let distance = delta.length();

        let new_distance = (distance - amount).clamp(MIN_ZOOM, MAX_ZOOM);

        self.position = self.target + delta.normalize_or_zero() * new_distance;
    }

    pub fn projection_matrix(&self) -> Mat4 {
        rh::proj::directx::perspective(self.fovy, self.aspect, self.near, self.far)
            * rh::view::look_at_mat4(self.position, self.target, self.up)
    }
}
