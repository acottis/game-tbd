use glam::Vec3;

use crate::graphics::ModelTransform;

use crate::physics::GRAVITY;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ModelId {
    Foo,
    Cube,
    Ground,
}

pub struct Animation {
    pub current_time: f32,
    duration: f32,
}

impl Animation {
    pub const fn new(duration: f32) -> Self {
        Self {
            current_time: 0.0,
            duration,
        }
    }
}

pub struct Entity {
    position: Vec3,
    scale: Vec3,
    physics: bool,
    falling: bool,
    pub model: ModelId,
    pub animation: Option<Animation>,
    // TODO: Leaky abstraction
    pub transform: ModelTransform,
}

impl Entity {
    pub fn new(
        position: Vec3,
        scale: Vec3,
        physics: bool,
        model: ModelId,
        transform: ModelTransform,
    ) -> Self {
        Self {
            position,
            scale,
            physics,
            falling: false,
            animation: None,
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
    pub const fn move_x(&mut self, distance: f32) {
        self.position.x += distance;
    }
    pub const fn move_y(&mut self, distance: f32) {
        self.position.y += distance;
    }
    pub const fn move_z(&mut self, distance: f32) {
        self.position.z += distance;
    }
    pub fn move_direction(&mut self, distance: f32, direction: Vec3) {
        self.position += direction * distance;
    }

    pub const fn jump(&mut self, distance: f32) {
        if self.falling {
            return;
        };
        self.move_y(distance);
        self.falling = true;
        self.animation = Some(Animation::new(1.0));
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

    const fn animate(&mut self, delta_time: f32) {
        if let Some(animation) = &mut self.animation {
            animation.current_time += delta_time;

            if animation.current_time >= animation.duration {
                self.animation = None
            }
        }
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
            entity.animate(delta_time);

            if entity.physics {
                entity.apply_gravity(delta_time);
                entity.check_collision();
            }
        }
    }
}
