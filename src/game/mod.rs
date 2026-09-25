pub mod animation;

use std::f32::consts::PI;

use glam::{Mat4, Quat, Vec3};

use glyphon::Color;
use winit::keyboard::KeyCode;

use crate::{
    assets::{AssetModelSet, ModelId},
    engine::{
        camera::Camera,
        light::Light,
        physics::{BoundingBox, GRAVITY, GroundCollision},
        store::{Handle, Store},
        text::{self, Text},
    },
    game::animation::{Animation, AnimationId},
    input::Input,
};

#[inline(always)]
fn transform(position: Vec3, rotation: Quat, scale: Vec3) -> Mat4 {
    Mat4::from_translation(position) * Mat4::from_quat(rotation) * Mat4::from_scale(scale)
}

pub struct Entity {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    velocity: Vec3,
    speed: f32,
    grounded: bool,
    bounding_box: BoundingBox,
    pub animation: Animation,
    pub nameplate: Option<Handle<Text>>,
    pub model: ModelId,
}

impl Entity {
    pub fn new(assets: &AssetModelSet, model: ModelId, position: Vec3, scale: Vec3) -> Self {
        let asset = assets.get(model);
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale,
            velocity: Vec3::ZERO,
            grounded: false,
            nameplate: None,
            model,
            bounding_box: asset.bounding_box,
            animation: Animation::new(asset),
            speed: 4.0,
        }
    }

    fn set_nameplate(&mut self, nameplate: Handle<Text>) {
        self.nameplate = Some(nameplate)
    }

    pub const fn position(&self) -> Vec3 {
        self.position
    }

    pub fn move_direction(&mut self, rotation_factor: f32, direction: Vec3) {
        self.velocity.z = direction.z * self.speed;
        self.velocity.x = direction.x * self.speed;

        if direction.length_squared() > 0.0 {
            let angle = direction.x.atan2(direction.z);
            let target = Quat::from_rotation_y(angle);

            self.rotation = self.rotation.slerp(target, rotation_factor);
        }
    }

    fn jump(&mut self, velocity: f32) {
        if !self.grounded {
            return;
        }

        self.grounded = false;
        self.velocity.y = velocity;
        self.animation.play(AnimationId::Jump);
    }

    #[inline(always)]
    fn bounds(&self) -> BoundingBox {
        self.bounding_box.transform(self.transform())
    }

    fn check_terrain_collision(&mut self, terrain: &GroundCollision) {
        // None means OOB
        let Some(height) = terrain.height_at(self.position) else {
            log::warn!("You are out of bounds!");
            return;
        };

        let bounds = self.bounds();
        if self.velocity.y <= 0.0 && bounds.min.y <= height {
            self.position.y += height - bounds.min.y;
            self.ground();
        }
    }

    fn ground(&mut self) {
        self.velocity.y = 0.0;
        self.grounded = true;

        // TODO: This needs some thought
        if self.animation.id() == AnimationId::Jump {
            self.animation.play(AnimationId::Idle);
        }
    }

    #[inline(always)]
    fn apply_gravity(&mut self, delta_time: f32) {
        self.velocity += GRAVITY * delta_time;
    }

    #[inline(always)]
    pub fn transform(&self) -> Mat4 {
        transform(self.position, self.rotation, self.scale)
    }

    pub fn move_and_collide(
        &mut self,
        delta_time: f32,
        ground: &GroundCollision,
        objects: &[Object],
    ) {
        // Assume we are not grounded until proven otherwise
        self.grounded = false;

        self.apply_gravity(delta_time);
        let movement = self.velocity * delta_time;
        let bounds = self.bounds();
        for object in objects {
            // TODO: We don't handle moving into the object before collision kicks in
            if let Some((time, normal)) = bounds.sweep(movement, &object.bounds()) {
                // println!("COLLISION time={time}, normal={normal:?}, movement={movement:?}");
                //
                // Moving into object from Above
                if normal.y == 1.0 && movement.y < 0.0 {
                    self.ground();
                }
                // Moving into object from below
                else if normal.y == -1.0 {
                    self.velocity.y = 0.0;
                } else if normal.z != 0.0 {
                    self.velocity.z = 0.0;
                } else if normal.x != 0.0 {
                    self.velocity.x = 0.0;
                }
            }
        }
        self.position += self.velocity * delta_time;
        self.check_terrain_collision(&ground);
    }
}

pub struct Terrain {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    collider: GroundCollision,
    pub model: ModelId,
}

impl Terrain {
    pub fn new(assets: &AssetModelSet, model: ModelId, position: Vec3, scale: Vec3) -> Self {
        let rotation = Quat::IDENTITY;

        let transform = transform(position, rotation, scale);
        let asset = assets.get(model);
        let collider = GroundCollision::new(asset, transform);
        Self {
            position,
            rotation,
            scale,
            model,
            collider,
        }
    }

    #[inline(always)]
    pub fn transform(&self) -> Mat4 {
        transform(self.position, self.rotation, self.scale)
    }
}

pub struct Object {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    bounding_box: BoundingBox,
    pub model: ModelId,
}
impl Object {
    pub fn new(assets: &AssetModelSet, model: ModelId, position: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale,
            model,
            bounding_box: assets.get(model).bounding_box,
        }
    }

    #[inline(always)]
    fn bounds(&self) -> BoundingBox {
        self.bounding_box.transform(self.transform())
    }

    #[inline(always)]
    pub fn transform(&self) -> Mat4 {
        transform(self.position, self.rotation, self.scale)
    }
}

pub struct Game {
    pub entities: Vec<Entity>,
    pub objects: Vec<Object>,
    pub terrain: Terrain,
    pub camera: Camera,
    pub light: Light,
    pub input: Input,
    pub texts: Store<Text>,
    pub fps: Handle<Text>,
}

impl Game {
    pub fn new(assets: &AssetModelSet) -> Self {
        let input = Input::new();
        let camera = Camera::new(&winit::dpi::PhysicalSize {
            width: 800,
            height: 600,
        });

        let light = Light::new(
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            0.8,
            0.1,
        );

        let mut player = Entity::new(assets, ModelId::Sabine, Vec3::ZERO, Vec3::ONE);
        let platform = Object::new(assets, ModelId::Platform, Vec3::Y * 3.0, Vec3::ONE);
        let terrain = Terrain::new(
            assets,
            ModelId::Ground,
            Vec3::ZERO,
            Vec3::new(100.0, 75.0, 100.0),
        );

        let mut texts = Store::new();

        let player_nameplate = texts.create(Text::new(
            Color::rgb(255, 255, 255),
            text::Anchor::Left,
            "Foo Nameplate 🦀",
        ));
        player.set_nameplate(player_nameplate);

        let fps = texts.create(Text::new(
            Color::rgb(255, 255, 255),
            text::Anchor::Right,
            "FPS: 0.0",
        ));

        let entities = vec![player];
        let objects = vec![platform];
        Self {
            entities,
            terrain,
            camera,
            light,
            input,
            objects,
            texts,
            fps,
        }
    }

    pub fn handle_inputs(&mut self, delta_time: f32) -> bool {
        let player = &mut self.entities[0];
        let camera = &mut self.camera;

        // Movement is relative to camera direction
        let mut movement = Vec3::ZERO;
        if self.input.is_pressed(KeyCode::KeyW) {
            movement += camera.forward_planar();
        }
        if self.input.is_pressed(KeyCode::KeyS) {
            movement -= camera.forward_planar()
        }
        if self.input.is_pressed(KeyCode::KeyA) {
            movement -= camera.right()
        }
        if self.input.is_pressed(KeyCode::KeyD) {
            movement += camera.right()
        }
        if self.input.is_pressed(KeyCode::Space) {
            player.jump(8.0);
        }
        if self.input.is_pressed(KeyCode::Digit0) {
            player.animation.play_loop(AnimationId::Wave)
        }
        if self.input.is_pressed(KeyCode::KeyU) {
            camera.rotate_pitch(delta_time * PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyJ) {
            camera.rotate_pitch(delta_time * -PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyH) {
            camera.rotate_yaw(delta_time * -PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::KeyK) {
            camera.rotate_yaw(delta_time * PI / 2.0)
        }
        if self.input.is_pressed(KeyCode::Escape) {
            return true;
        }
        let rotation_factor = (10.0 * delta_time).min(1.0);
        player.move_direction(rotation_factor, movement);
        camera.follow(player.position());
        false
    }

    pub fn update(&mut self, delta_time: f32, assets: &AssetModelSet) {
        self.texts
            .get_mut(self.fps)
            .unwrap()
            .set_text(format!("FPS: {:.0}", 1.0 / delta_time));

        for entity in &mut self.entities {
            if entity.velocity == Vec3::ZERO && entity.animation.id() == AnimationId::Walk {
                entity.animation.play_loop(AnimationId::Idle);
            }
            if entity.velocity != Vec3::ZERO {
                entity.animation.play_loop(AnimationId::Walk);
            }
            let asset = assets.get(entity.model);
            entity.animation.update(delta_time, asset);
            entity.move_and_collide(delta_time, &self.terrain.collider, &self.objects);
        }
    }
}
