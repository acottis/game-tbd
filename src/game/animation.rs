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
    layers: Layers,
}
impl Animation {
    pub fn new(asset: &Asset) -> Self {
        Self {
            layers: Layers::new(),
            skin_poses: asset.skins.iter().map(SkinPose::new).collect(),
            pose: Pose::new(asset),
            scratch_pose: Pose::new(asset),
        }
    }

    #[inline(always)]
    pub fn add_layer(&mut self, id: AnimationId, weight: f32, looping: bool) {
        self.layers.add(id, weight, looping)
    }

    pub fn crossfade_loop(&mut self, id: AnimationId, duration: f32) {
        self.layers.base.crossfade_to(id, true, duration);
    }

    // TOOD: War crime perf
    pub fn update(&mut self, delta_time: f32, asset: &Asset) {
        let base = &mut self.layers.base;

        let clip = asset.animations.get(base.id);

        // TODO: Avoiding the borrowchecker lol
        if let Some(clip) = clip {
            base.update(delta_time, clip);
        }

        match base.crossfade {
            Some(ref mut crossfade) => {
                crossfade.update(delta_time);

                // Previous animation. Fallback is rest pose
                if let Some(previous_clip) = asset.animations.get(crossfade.id) {
                    self.pose.sample(asset, previous_clip, crossfade.time);
                } else {
                    log::warn!("1. Base Clip missing: {:?}", base.id);
                    self.pose.sample_rest(asset);
                }

                // Previous animation. Fallback is rest pose
                if let Some(clip) = clip {
                    self.scratch_pose.sample(asset, clip, base.time);
                } else {
                    log::warn!("2. Base Clip missing: {:?}", base.id);
                    self.scratch_pose.sample_rest(asset);
                }

                self.pose.blend(&self.scratch_pose, crossfade.weight());
                if crossfade.finished() {
                    base.crossfade = None;
                }
            }
            None => {
                if let Some(clip) = clip {
                    self.pose.sample(asset, clip, base.time);
                } else {
                    log::warn!("3. Base Clip missing: {:?}", base.id);
                    self.pose.sample_rest(asset);
                }
            }
        }

        // Blend layers into base
        for layer in self.layers.iter_mut() {
            if let Some(clip) = asset.animations.get(layer.id) {
                layer.update(delta_time, clip);
                self.scratch_pose.sample(asset, clip, layer.time);
                self.pose.blend(&self.scratch_pose, layer.weight);
            };
        }

        // Remove finished layers
        self.layers.inner.retain(|layer| !layer.finished(asset));

        // Update the pose and then joints
        self.pose.update(asset);
        for (skin, skin_pose) in asset.skins.iter().zip(&mut self.skin_poses) {
            skin_pose.update(skin, &self.pose);
        }
    }
}

#[derive(Debug)]
struct Layers {
    base: Layer,
    inner: Vec<Layer>,
}
impl Layers {
    fn new() -> Self {
        let base = Layer::new(AnimationId::Idle, 1.0, true);
        Self {
            base,
            inner: Vec::with_capacity(4),
        }
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Layer> {
        self.inner.iter_mut()
    }

    #[inline(always)]
    fn add(&mut self, id: AnimationId, weight: f32, looping: bool) {
        if let Some(layer) = self.inner.iter_mut().find(|layer| layer.id == id) {
            layer.time = 0.0;
            layer.looping = looping;
            layer.weight = weight;
            return;
        }

        self.inner.push(Layer::new(id, weight, looping));
    }
}

#[derive(Debug, Clone, Copy)]
struct Crossfade {
    id: AnimationId,
    time: f32,
    progress: f32,
    duration: f32,
}

impl Crossfade {
    #[inline(always)]
    fn update(&mut self, delta_time: f32) {
        self.progress += delta_time;
        self.time += delta_time;
    }

    #[inline(always)]
    fn weight(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.progress / self.duration).clamp(0.0, 1.0)
        }
    }

    #[inline(always)]
    fn finished(&self) -> bool {
        self.progress >= self.duration
    }
}

#[derive(Debug, Clone, Copy)]
struct Layer {
    id: AnimationId,
    time: f32,
    weight: f32,
    looping: bool,
    crossfade: Option<Crossfade>,
}

impl Layer {
    #[inline(always)]
    fn new(id: AnimationId, weight: f32, looping: bool) -> Self {
        Self {
            id,
            time: 0.0,
            weight,
            looping,
            crossfade: None,
        }
    }

    #[inline(always)]
    fn finished(&self, asset: &Asset) -> bool {
        let Some(clip) = asset.animations.get(self.id) else {
            return true;
        };

        !self.looping && self.time >= clip.duration()
    }

    #[inline(always)]
    fn update(&mut self, delta_time: f32, clip: &AnimationClip) {
        self.time += delta_time;

        if self.looping && clip.duration() > 0.0 {
            self.time %= clip.duration();
        }
    }

    fn crossfade_to(&mut self, id: AnimationId, looping: bool, duration: f32) {
        // Ignore requests to crossfade to the current animation being played
        if self.id == id {
            return;
        }

        // Ignore additional requests to crossfade to current target animation
        if let Some(crossfade) = self.crossfade {
            if crossfade.id == id {
                return;
            }
        }

        self.crossfade = Some(Crossfade {
            id: self.id,
            time: self.time,
            progress: 0.0,
            duration,
        });

        self.id = id;
        self.time = 0.0;
        self.looping = looping;
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
