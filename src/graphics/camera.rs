use std::f32::consts::PI;

use winit::dpi::PhysicalSize;

use crate::maths::{Mat3, Mat4, Vec3, Vec4};

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

    pub fn follow(&mut self, target: Vec3) {
        let old_target = self.target;
        let old_position = self.position;

        let offset = old_position - old_target;

        self.target = target;
        self.position = target + offset;
    }
    pub fn forward_direction(&self) -> Vec3 {
        let mut forward = self.target - self.position;
        forward.y = 0.0;
        forward.normalise()
    }
    pub fn right_direction(&self) -> Vec3 {
        let forward = self.forward_direction();
        forward.cross(&self.up).normalise()
    }

    // pub fn rotate_x(&mut self, delta_time: f32, theta: f32) {
    //     let rotation = Mat3::rotation_x(theta * delta_time);
    //     let offset = self.position - self.target;

    //     self.position = self.target + rotation * offset;
    // }

    // pub fn rotate_y(&mut self, delta_time: f32, theta: f32) {
    //     let rotation = Mat3::rotation_y(theta * delta_time);
    //     let offset = self.position - self.target;

    //     self.position = self.target + rotation * offset;
    // }

    // pub fn rotate_z(&mut self, delta_time: f32, theta: f32) {
    //     let rotation = Mat3::rotation_z(theta * delta_time);
    //     let offset = self.position - self.target;

    //     self.position = self.target + rotation * offset;
    // }

    // pub fn move_forward(&mut self, delta_time: f32, speed: f32) {
    //     let forward = (self.target - self.position).normalise();
    //     self.position += forward * speed * delta_time;
    // }

    // /// + is right, - is left
    // pub fn strafe(&mut self, delta_time: f32, speed: f32) {
    //     let forward = (self.target - self.position).normalise();
    //     let right = forward.cross(&self.up).normalise();
    //     //let right = self.up.cross(&forward).normalise();

    //     let delta = right * speed * delta_time;

    //     self.position += delta;
    //     self.target += delta;
    // }

    pub fn zoom(&mut self, amount: f32) {
        let offset = self.position - self.target;
        let direction = offset.normalise();

        self.position -= direction * amount;
    }

    fn view_rh(&self) -> Mat4 {
        let forward = (self.target - self.position).normalise();
        let right = forward.cross(&self.up).normalise();
        let up = right.cross(&forward).normalise();

        let projection_x = -right.dot(&self.position);
        let projection_y = -up.dot(&self.position);
        let projection_z = forward.dot(&self.position);

        Mat4 {
            x: Vec4::new(right.x, up.x, -forward.x, 0.0),
            y: Vec4::new(right.y, up.y, -forward.y, 0.0),
            z: Vec4::new(right.z, up.z, -forward.z, 0.0),
            w: Vec4::new(projection_x, projection_y, projection_z, 1.0),
        }
    }

    fn perspective_rh(&self) -> Mat4 {
        let tan_half_fov = 1.0 / (self.fovy / 2.0).tan();
        let range = self.far - self.near;
        let depth = -(self.far + self.near) / range;
        let project = -(2.0 * self.far * self.near) / range;
        Mat4 {
            x: Vec4::new(tan_half_fov / self.aspect, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, tan_half_fov, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, depth, -1.0),
            w: Vec4::new(0.0, 0.0, project, 0.0),
        }
    }

    pub fn view_perspective_rh(&self) -> Mat4 {
        self.view_rh() * self.perspective_rh()
    }

    pub fn set_aspect_ratio(&mut self, size: &PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32
    }
}
