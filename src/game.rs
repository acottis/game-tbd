use crate::graphics::ModelTransform;

use crate::maths::Vec3;
use crate::physics::GRAVITY;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ModelId {
    Foo,
    Cube,
    Ground,
}

pub struct Entity {
    position: Vec3,
    scale: Vec3,
    physics: bool,
    falling: bool,
    pub model: ModelId,
    pub transform: ModelTransform,
}

impl Entity {
    pub fn new(
        position: Vec3,
        scale: Vec3,
        physics: bool,
        model: ModelId,
        // TODO: Leaky abstraction
        transform: ModelTransform,
    ) -> Self {
        Self {
            position,
            scale,
            physics,
            falling: false,
            model,
            transform,
        }
    }
    pub const fn position(&self) -> Vec3 {
        self.position
    }
    pub const fn scale(&self) -> Vec3 {
        self.scale
    }
    pub const fn move_x(&mut self, delta_time: f32, x: f32) {
        self.position.x += x * delta_time;
    }
    pub const fn move_y(&mut self, delta_time: f32, y: f32) {
        self.position.y += y * delta_time;
    }
    pub const fn move_z(&mut self, delta_time: f32, z: f32) {
        self.position.z += z * delta_time;
    }
    pub fn move_direction(&mut self, delta_time: f32, speed: f32, direction: Vec3) {
        self.position += direction * speed * delta_time;
    }

    pub const fn jump(&mut self, delta_time: f32, y: f32) {
        if self.falling {
            return;
        };
        self.move_y(delta_time, y);
        self.falling = true
    }

    const fn check_collision(&mut self) {
        if self.position.y <= 0.0 {
            self.falling = false;
            self.position.y = 0.0;
        }
    }
    fn apply_gravity(&mut self, delta_time: f32) {
        self.position += Vec3::new(0.0, GRAVITY, 0.0) * delta_time;
    }
}

pub struct Game {
    pub entities: Vec<Entity>,
}
impl Game {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in self.entities.iter_mut() {
            if entity.physics {
                entity.apply_gravity(delta_time);
                entity.check_collision();
            }
        }
    }
}
