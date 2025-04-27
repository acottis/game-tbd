use crate::graphics::Asset;
use crate::math::{Mat4, Vec3};

pub struct Entity {
    coords: Vec3,
    scale: Vec3,
    pub asset: Asset,
}

impl Entity {
    pub fn new(coords: Vec3, asset: Asset) -> Self {
        Self {
            coords,
            scale: Vec3::xyz(1.0),
            asset,
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
        entities.push(Entity::new(Vec3::zeroes(), Asset::Ground));
        entities.push(Entity::new(Vec3::y(), Asset::Cube));
        Self { entities }
    }
}
