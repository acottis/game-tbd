use crate::graphics::Asset;
use crate::math::{Mat4, Vec3};
use crate::physics::GRAVITY;

pub struct Entity {
    position: Vec3,
    scale: Vec3,
    physics: bool,
    pub asset: Asset,
}

impl Entity {
    pub fn new(coords: Vec3, asset: Asset, physics: bool) -> Self {
        Self {
            position: coords,
            scale: Vec3::xyz(1.0),
            asset,
            physics,
        }
    }

    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.position) * Mat4::from_scaling(self.scale)
    }

    fn check_collision(&mut self) {
        if self.position.y <= 0.0 {
            self.position.y = 0.0;
        }
    }
}

pub struct Game {
    pub entities: Vec<Entity>,
}
impl Game {
    pub fn new() -> Self {
        let mut entities = Vec::new();
        let mut cube = Entity::new(Vec3::new(0.0, 2.0, 0.0), Asset::Cube, true);
        cube.scale = Vec3::xyz(0.3);
        entities.push(Entity::new(Vec3::zeroes(), Asset::Ground, false));
        entities.push(cube);
        Self { entities }
    }

    pub fn update(&mut self, delta_time: f32) {
        for entity in self.entities.iter_mut() {
            if entity.physics {
                entity.position += Vec3::new(0.0, GRAVITY, 0.0) * delta_time;
                entity.check_collision();
            }
        }
    }
}
