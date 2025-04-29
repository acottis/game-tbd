use crate::graphics::{MeshInstance, MeshInstanceId, State};
use crate::math::{Mat4, Vec3};
use crate::physics::GRAVITY;

pub struct Entity {
    position: Vec3,
    scale: Vec3,
    physics: bool,
    pub mesh: MeshInstance,
}

impl Entity {
    pub fn new(coords: Vec3, mesh: MeshInstance, physics: bool) -> Self {
        Self {
            position: coords,
            scale: Vec3::xyz(1.0),
            physics,
            mesh,
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

    pub fn load(&mut self, state: &State) {
        let ground_mesh = state.mesh_instance(MeshInstanceId::Ground);
        let cube_mesh1 = state.mesh_instance(MeshInstanceId::Cube);
        let cube_mesh2 = state.mesh_instance(MeshInstanceId::Cube);
        let ground = Entity::new(Vec3::zeroes(), ground_mesh, false);
        let mut cube1 =
            Entity::new(Vec3::new(-1.0, 3.0, 0.0), cube_mesh1, true);
        cube1.scale = Vec3::xyz(0.3);
        let mut cube2 = Entity::new(Vec3::new(1.0, 3.0, 0.0), cube_mesh2, true);
        cube2.scale = Vec3::xyz(0.3);
        self.entities.push(ground);
        self.entities.push(cube2);
        self.entities.push(cube1);
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
