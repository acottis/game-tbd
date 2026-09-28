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

#[derive(Debug, Clone, Copy)]
struct Playback {
    id: AnimationId,
    looping: bool,
    speed: f32,
    time: f32,
}

impl Playback {
    fn new(id: AnimationId, looping: bool, speed: f32) -> Self {
        Self {
            id,
            time: 0.0,
            looping,
            speed,
        }
    }

    #[inline(always)]
    fn update(&mut self, delta_time: f32, duration: f32) {
        self.time += delta_time * self.speed;

        if self.looping && duration > 0.0 && self.time >= duration {
            self.time %= duration;
        }
    }

    #[inline(always)]
    fn finished(&self, duration: f32) -> bool {
        !self.looping && self.time >= duration
    }
}

pub struct Animation {
    pub skin_poses: Vec<SkinPose>,
    pub pose: Pose,
    next_pose: Pose,
    current: Playback,
    crossfade: Option<Crossfade>,
    layers: Vec<Layer>,
}
impl Animation {
    pub fn new(asset: &Asset) -> Self {
        Self {
            layers: Vec::with_capacity(8),
            skin_poses: asset.skins.iter().map(SkinPose::new).collect(),
            pose: Pose::new(asset),
            next_pose: Pose::new(asset),
            current: Playback::new(AnimationId::Idle, true, 1.0),
            crossfade: None,
        }
    }

    #[inline(always)]
    pub fn add_layer(&mut self, id: AnimationId, weight: f32, speed: f32, looping: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.playback.id == id) {
            layer.playback.time = 0.0;
            layer.playback.looping = looping;
            layer.playback.speed = speed;
            layer.weight = weight;
        } else {
            self.layers.push(Layer::new(id, weight, speed, looping));
        }
    }

    pub fn crossfade_loop(&mut self, id: AnimationId, duration: f32, speed: f32) {
        // Ignore requests to crossfade to the current animation being played
        if self.current.id == id {
            return;
        }

        // Ignore additional requests to crossfade to current target animation
        if let Some(crossfade) = self.crossfade {
            if crossfade.to.id == id {
                return;
            }
        }

        self.crossfade = Some(Crossfade {
            to: Playback::new(id, true, speed),
            duration,
            elapsed: 0.0,
        });
    }

    // TOOD: War crime perf with sample_rest not being cached
    // many copies for fun on slightly happier paths but I
    // dont see what I can do about it.
    fn update_current(&mut self, delta_time: f32, asset: &Asset) {
        if let Some(clip) = asset.animations.get(self.current.id) {
            self.current.update(delta_time, clip.duration());
            self.pose.sample(asset, clip, self.current.time);
        } else {
            // Short cicuit when there is no clips's to sample
            if self.crossfade.is_none() && self.layers.is_empty() {
                return;
            }
            self.pose.sample_rest(asset);
        }

        if let Some(ref mut crossfade) = self.crossfade {
            if let Some(target_clip) = asset.animations.get(crossfade.to.id) {
                crossfade.update(delta_time, target_clip.duration());
                self.next_pose.sample(asset, target_clip, crossfade.to.time);
            } else {
                crossfade.update(delta_time, 0.0);
                self.next_pose.sample_rest(asset);
            }

            self.pose.blend(&self.next_pose, crossfade.weight());

            if crossfade.finished() {
                self.current = crossfade.to;
                self.crossfade = None;
            }
        }
    }

    pub fn update(&mut self, delta_time: f32, asset: &Asset) {
        self.update_current(delta_time, asset);

        // Update Layers and Remove finished
        self.layers.retain_mut(|layer| {
            let Some(clip) = asset.animations.get(layer.playback.id) else {
                return false;
            };
            layer.playback.update(delta_time, clip.duration());
            self.next_pose.sample(asset, clip, layer.playback.time);
            self.pose.blend(&self.next_pose, layer.weight);

            !layer.playback.finished(clip.duration())
        });

        // Update the pose and then joints
        self.pose.update(asset);
        for (skin, skin_pose) in asset.skins.iter().zip(&mut self.skin_poses) {
            skin_pose.update(skin, &self.pose);
        }
    }

    #[inline(always)]
    pub fn set_speed(&mut self, speed: f32) {
        self.current.speed = speed.max(0.0);

        if let Some(crossfade) = &mut self.crossfade {
            crossfade.to.speed = speed.max(0.0);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Crossfade {
    to: Playback,
    elapsed: f32,
    duration: f32,
}

impl Crossfade {
    #[inline(always)]
    fn update(&mut self, delta_time: f32, duration: f32) {
        self.elapsed += delta_time;
        self.to.update(delta_time, duration);
    }

    #[inline(always)]
    fn weight(&self) -> f32 {
        if self.duration == 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    #[inline(always)]
    fn finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

#[derive(Debug, Clone, Copy)]
struct Layer {
    playback: Playback,
    weight: f32,
}

impl Layer {
    #[inline(always)]
    fn new(id: AnimationId, weight: f32, speed: f32, looping: bool) -> Self {
        Self {
            playback: Playback::new(id, looping, speed),
            weight,
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
