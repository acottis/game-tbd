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
    state: State,
    pub pose: Pose,
    pub skin_poses: Vec<SkinPose>,
}
impl Animation {
    pub fn new(asset: &Asset) -> Self {
        Self {
            state: State::new(AnimationId::Idle),
            skin_poses: asset.skins.iter().map(SkinPose::new).collect(),
            pose: Pose::new(&asset.rest_world_transforms),
        }
    }

    #[inline(always)]
    pub fn play(&mut self, id: AnimationId) {
        self.state.play(id, false);
    }

    #[inline(always)]
    pub fn play_loop(&mut self, id: AnimationId) {
        self.state.play(id, true);
    }

    #[inline(always)]
    pub fn id(&self) -> AnimationId {
        self.state.id
    }

    pub fn update(&mut self, delta_time: f32, asset: &Asset) {
        self.state.update(delta_time);

        // If the asset has the AnimateID available carry on
        let Some(mut clip) = asset.animations.get(self.state.id) else {
            // AnimationClip is missing
            // But its not dirty so do nothing
            if !self.state.dirty {
                // If its dirty update to default pose
                self.reset_pose(asset);
                // Prevent this running every frame
                self.state.dirty = true;
            }
            return;
        };

        if !self.state.looping && self.state.current_time >= clip.duration() {
            self.state.id = AnimationId::Idle;
            self.state.current_time = 0.0;

            // Update immediately to avoid doing the old animation
            // for an additional frame
            let Some(idle_clip) = asset.animations.get(self.state.id) else {
                self.reset_pose(asset);
                return;
            };

            clip = idle_clip;
        }

        self.pose.update(&asset, clip, self.state.current_time);

        for (skin, skin_pose) in asset.skins.iter().zip(&mut self.skin_poses) {
            skin_pose.update(skin, &self.pose);
        }
    }

    fn reset_pose(&mut self, asset: &Asset) {
        self.pose
            .world_transforms
            .copy_from_slice(&asset.rest_world_transforms);

        for (skin, skin_pose) in asset.skins.iter().zip(&mut self.skin_poses) {
            skin_pose.update(skin, &self.pose);
        }
    }
}

struct State {
    id: AnimationId,
    current_time: f32,
    looping: bool,
    /// Tracks if we change [AnimationId] to stop us repeatidly
    /// reseting on an asset not having the [AnimationClip] for
    /// that [AnimationId]
    dirty: bool,
}

impl State {
    const fn new(id: AnimationId) -> Self {
        Self {
            id,
            current_time: 0.0,
            dirty: false,
            looping: false,
        }
    }

    fn play(&mut self, id: AnimationId, looping: bool) {
        if self.id == id {
            return;
        }

        self.id = id;
        self.current_time = 0.0;
        self.dirty = false;
        self.looping = looping
    }

    fn update(&mut self, delta_time: f32) {
        self.current_time += delta_time
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
