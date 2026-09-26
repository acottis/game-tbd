use crate::{
    assets::Asset,
    engine::animation::{AnimationClip, Pose, SkinPose},
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum AnimationId {
    Idle = 0,
    Walk,
    Jump,
    Wave,
}

impl AnimationId {
    const COUNT: usize = 4;
}

impl TryFrom<&str> for AnimationId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "idle" => Ok(Self::Idle),
            "walk" => Ok(Self::Walk),
            "jump" => Ok(Self::Jump),
            "wave" => Ok(Self::Wave),
            _ => Err(format!("Invalid Animation ID: {value}")),
        }
    }
}

pub struct Animation {
    pub skin_poses: Vec<SkinPose>,
    pub pose: Pose,
    scratch_pose: Pose,
    layers: Vec<Layer>,
}
impl Animation {
    pub fn new(asset: &Asset) -> Self {
        let mut layers = Vec::with_capacity(4);
        layers.push(Layer::new(AnimationId::Idle, 1.0, true));
        Self {
            layers,
            skin_poses: asset.skins.iter().map(SkinPose::new).collect(),
            pose: Pose::new(asset),
            scratch_pose: Pose::new(asset),
        }
    }

    #[inline(always)]
    pub fn play(&mut self, id: AnimationId) {
        if self.layers[0].id == id {
            return;
        }
        self.layers[0] = Layer::new(id, 1.0, false);
    }

    #[inline(always)]
    pub fn add_layer(&mut self, id: AnimationId, looping: bool, weight: f32) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.time = 0.0;
            layer.looping = looping;
            layer.weight = weight;
            return;
        }

        self.layers.push(Layer::new(id, weight, looping));
    }

    #[inline(always)]
    pub fn play_loop(&mut self, id: AnimationId) {
        if self.layers[0].id == id {
            return;
        }
        self.layers[0] = Layer::new(id, 1.0, true);
    }

    #[inline(always)]
    pub fn is_playing(&self, id: AnimationId) -> bool {
        self.layers.iter().any(|layer| layer.id == id)
    }

    pub fn update(&mut self, delta_time: f32, asset: &Asset) {
        for layer in &mut self.layers {
            if let Some(clip) = asset.animations.get(layer.id) {
                layer.update(delta_time, clip);
            };
        }

        let base = &mut self.layers[0];
        if let Some(clip) = asset.animations.get(base.id) {
            self.pose.sample(asset, clip, base.time);
        } else if base.needs_reset {
            log::warn!("Base Clip not found {:?}", base.id);
            self.pose.reset_to_rest(asset);
            base.needs_reset = false;
        };

        for layer in self.layers.iter().skip(1) {
            if let Some(clip) = asset.animations.get(layer.id) {
                self.scratch_pose.sample(asset, clip, layer.time);
                self.pose.blend_into(&self.scratch_pose, layer.weight);
            }
        }

        self.pose.update(asset);

        for (skin, skin_pose) in asset.skins.iter().zip(&mut self.skin_poses) {
            skin_pose.update(skin, &self.pose);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Layer {
    id: AnimationId,
    time: f32,
    weight: f32,
    looping: bool,
    // TODO: Think about this
    needs_reset: bool,
}

impl Layer {
    fn new(id: AnimationId, weight: f32, looping: bool) -> Self {
        Self {
            id,
            time: 0.0,
            weight,
            looping,
            needs_reset: true,
        }
    }

    fn update(&mut self, delta_time: f32, clip: &AnimationClip) {
        self.time += delta_time;

        if self.looping && clip.duration() > 0.0 {
            self.time %= clip.duration();
        }
    }
}

#[derive(Debug)]
pub struct AnimationSet([Option<AnimationClip>; AnimationId::COUNT]);

impl AnimationSet {
    pub fn new() -> Self {
        Self(std::array::from_fn(|_| None))
    }

    pub fn get(&self, id: AnimationId) -> Option<&AnimationClip> {
        self.0[id as usize].as_ref()
    }

    pub fn insert(&mut self, id: AnimationId, clip: AnimationClip) {
        self.0[id as usize] = Some(clip)
    }
}
