use glam::{Mat4, Vec3};

use physics::GRAVITY;

use crate::{
    assets::{AssetModel, AssetModels, ModelId},
    game::{
        camera::Camera,
        light::Light,
        physics::{BoundingBox, GroundCollision},
    },
};

pub mod animation;
mod camera;
pub mod input;
pub mod light;
mod physics;

#[inline(always)]
fn transform(position: Vec3, scale: Vec3) -> Mat4 {
    Mat4::from_translation(position) * Mat4::from_scale(scale)
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
    velocity: Vec3,
    scale: Vec3,
    falling: bool,
    bounding_box: BoundingBox,
    pub animation: Option<Animation>,
    pub model: ModelId,
}

impl Entity {
    pub fn new(model: ModelId, position: Vec3, scale: Vec3, bounding_box: BoundingBox) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            scale,
            falling: false,
            animation: None,
            model,
            bounding_box,
        }
    }
    pub const fn position(&self) -> Vec3 {
        self.position
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
        // TODO: Hacked in infinite jump animation
        self.animation = Some(Animation::new(1000.0));
    }

    fn bounds(&self) -> BoundingBox {
        self.bounding_box.transform(self.transform())
    }

    fn check_collision(&mut self, terrain: &GroundCollision) {
        // None means OOB
        let Some(height) = terrain.height_at(self.position) else {
            return;
        };

        if self.velocity.y <= 0.0 && self.bounds().min.y <= height {
            self.position.y += height - self.bounds().min.y;
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

    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.position) * Mat4::from_scale(self.scale)
    }
}

pub struct Terrain {
    position: Vec3,
    scale: Vec3,
    collider: GroundCollision,
    pub model: ModelId,
}

impl Terrain {
    pub fn new(model: ModelId, position: Vec3, scale: Vec3, asset: &AssetModel) -> Self {
        let transform = transform(position, scale);
        let collider = GroundCollision::new(asset, transform);
        Self {
            position,
            scale,
            model,
            collider,
        }
    }
    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.position) * Mat4::from_scale(self.scale)
    }
}

pub struct Game {
    pub entities: Vec<Entity>,
    pub terrain: Terrain,
    pub camera: Camera,
    pub light: Light,
}

impl Game {
    pub fn new(assets: &AssetModels) -> Self {
        let camera = Camera::new(&winit::dpi::PhysicalSize {
            width: 800,
            height: 600,
        });
        let light = Light::new(Vec3::new(0.0, 0.5, 0.5), Vec3::new(1.0, 1.0, 1.0), 0.9);

        let cube_asset = assets.get(ModelId::Foo);
        let entities = vec![Entity::new(
            ModelId::Foo,
            Vec3::ZERO,
            Vec3::splat(0.3),
            BoundingBox::new(cube_asset),
        )];

        let ground_asset = assets.get(ModelId::Ground);
        let terrain = Terrain::new(ModelId::Ground, Vec3::ZERO, Vec3::splat(50.0), ground_asset);
        Self {
            entities,
            terrain,
            camera,
            light,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in &mut self.entities {
            entity.animate(delta_time);

            entity.apply_gravity(delta_time);
            entity.apply_velocity(delta_time);
            entity.check_collision(&self.terrain.collider);
        }
    }
}
