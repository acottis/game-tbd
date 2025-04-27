use crate::graphics::Asset;
use crate::math::{Mat4, Vec3};

pub struct Entity {
    coords: Vec3,
    scale: Vec3,
    physics: bool,
    pub asset: Asset,
}

impl Entity {
    pub fn new(coords: Vec3, asset: Asset, physics: bool) -> Self {
        Self {
            coords,
            scale: Vec3::xyz(1.0),
            asset,
            physics,
        }
    }

    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.coords) * Mat4::from_scaling(self.scale)
    }
}

pub struct Game {
    pub entities: Vec<Entity>,
}
impl Game {
    pub fn new() -> Self {
        let mut entities = Vec::new();
        entities.push(Entity::new(Vec3::zeroes(), Asset::Ground, false));
        entities.push(Entity::new(Vec3::y(), Asset::Cube, true));
        Self { entities }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in self.entities.iter_mut() {
            if entity.physics {
                entity.coords +=
                    Vec3::new(0.0, crate::physics::GRAVITY, 0.0) * delta_time;
            }
        }
    }
}
