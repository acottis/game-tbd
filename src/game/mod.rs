use glam::{Mat4, Vec3};

use physics::GRAVITY;

use crate::{
    assets::{AssetModels, ModelId},
    game::camera::Camera,
};

pub mod animation;
mod camera;
pub mod input;
mod physics;

pub struct Animation {
    current_time: f32,
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
    velocity: Vec3,
    scale: Vec3,
    falling: bool,
    animation: Option<Animation>,
    pub model: ModelId,
}

impl Entity {
    pub fn new(position: Vec3, scale: Vec3, model: ModelId) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            scale,
            falling: false,
            animation: None,
            model,
        }
    }
    pub fn move_direction(&mut self, distance: f32, direction: Vec3) {
        self.position += direction * distance;
    }

    pub const fn jump(&mut self, velocity: f32) {
        if self.falling {
            return;
        }

        self.velocity.y = velocity;
        self.falling = true;
        // TODO: Hacked in infinite jump
        self.animation = Some(Animation::new(1000.0));
    }

    pub const fn position(&self) -> Vec3 {
        self.position
    }
    const fn check_collision(&mut self) {
        if self.position.y <= 0.0 {
            self.position.y = 0.0;
            self.velocity.y = 0.0;

            self.falling = false;
            self.animation = None;
        }
    }

    fn apply_gravity(&mut self, delta_time: f32) {
        self.velocity += GRAVITY * delta_time;
    }

    fn apply_velocity(&mut self, delta_time: f32) {
        self.position += self.velocity * delta_time;
    }

    const fn animate(&mut self, delta_time: f32) {
        if let Some(animation) = &mut self.animation {
            animation.current_time += delta_time;

            if animation.current_time >= animation.duration {
                self.animation = None
            }
        }
    }

    pub fn transform(&self, models: &AssetModels) -> Mat4 {
        let transform = Mat4::from_translation(self.position) * Mat4::from_scale(self.scale);

        match self.animation {
            Some(ref animation) => {
                let clip = &models.get(self.model).animations[0];
                let (translation, rotation, scale) = clip.sample(animation.current_time);
                transform * Mat4::from_scale_rotation_translation(scale, rotation, translation)
            }
            None => transform,
        }
    }
}

pub struct Terrain {
    position: Vec3,
    scale: Vec3,
    pub model: ModelId,
}

impl Terrain {
    pub fn new(position: Vec3, scale: Vec3, model: ModelId) -> Self {
        Self {
            position,
            scale,
            model,
        }
    }
    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.position) * Mat4::from_scale(self.scale)
    }
}

pub struct Game {
    pub terrain: Terrain,
    pub entities: Vec<Entity>,
    pub camera: Camera,
}

impl Game {
    pub fn new() -> Self {
        let camera = Camera::new(&winit::dpi::PhysicalSize {
            width: 800,
            height: 600,
        });
        let mut entities = Vec::new();
        let cube = Entity::new(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(0.3), ModelId::Foo);
        entities.extend([cube]);

        Self {
            terrain: Terrain::new(Vec3::ZERO, Vec3::splat(100.0), ModelId::Ground),
            entities,
            camera,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in self.entities.iter_mut() {
            entity.animate(delta_time);

            entity.apply_gravity(delta_time);
            entity.apply_velocity(delta_time);
            entity.check_collision();
        }
    }
}
