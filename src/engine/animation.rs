use glam::{Mat4, Quat, Vec3};
use gltf::animation::Interpolation;

use crate::assets::{Asset, NodeId};

// TODO: Perf
fn keyframes(times: &[f32], time: f32) -> (usize, usize, f32) {
    if times.len() <= 1 {
        return (0, 0, 0.0);
    }

    if time <= times[0] {
        return (0, 0, 0.0);
    }

    let last = times.len() - 1;

    if time >= times[last] {
        return (last, last, 0.0);
    }

    for i in 0..last {
        if time >= times[i] && time < times[i + 1] {
            let duration = times[i + 1] - times[i];
            let t = (time - times[i]) / duration;

            return (i, i + 1, t);
        }
    }

    (last, last, 0.0)
}

fn sample_vec3(interpolation: Interpolation, times: &[f32], values: &[Vec3], time: f32) -> Vec3 {
    let (i0, i1, t) = keyframes(times, time);
    match interpolation {
        Interpolation::Step => values[i0],
        Interpolation::Linear => values[i0].lerp(values[i1], t),
        Interpolation::CubicSpline => todo!("Cubic spline"),
    }
}

fn sample_quat(interpolation: Interpolation, times: &[f32], values: &[Quat], time: f32) -> Quat {
    let (i0, i1, t) = keyframes(times, time);
    match interpolation {
        Interpolation::Step => values[i0],
        Interpolation::Linear => values[i0].slerp(values[i1], t),
        Interpolation::CubicSpline => todo!("Cubic spline"),
    }
}

#[derive(Debug, Clone)]
pub struct Translation {
    interpolation: Interpolation,
    times: Vec<f32>,
    values: Vec<Vec3>,
}

impl Translation {
    pub fn new(interpolation: Interpolation, times: Vec<f32>, values: Vec<Vec3>) -> Self {
        Self {
            interpolation,
            times,
            values,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rotation {
    interpolation: Interpolation,
    times: Vec<f32>,
    values: Vec<Quat>,
}

impl Rotation {
    pub fn new(interpolation: Interpolation, times: Vec<f32>, values: Vec<Quat>) -> Self {
        Self {
            interpolation,
            times,
            values,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scale {
    interpolation: Interpolation,
    times: Vec<f32>,
    values: Vec<Vec3>,
}

impl Scale {
    pub fn new(interpolation: Interpolation, times: Vec<f32>, values: Vec<Vec3>) -> Self {
        Self {
            interpolation,
            times,
            values,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NodeAnimation {
    pub translation: Option<Translation>,
    pub rotation: Option<Rotation>,
    pub scale: Option<Scale>,
}

#[derive(Debug)]
pub struct AnimationClip {
    nodes: Vec<Option<NodeAnimation>>,
    duration: f32,
}

impl AnimationClip {
    pub fn new(nodes: Vec<Option<NodeAnimation>>, duration: f32) -> Self {
        Self { nodes, duration }
    }
    pub fn duration(&self) -> f32 {
        self.duration
    }
}

impl AnimationClip {
    pub fn sample_node(&self, index: u32, base_transform: Mat4, time: f32) -> Mat4 {
        let Some(ref node) = self.nodes[index as usize] else {
            return base_transform;
        };

        let (mut translation, mut rotation, mut scale) =
            base_transform.to_scale_rotation_translation();

        if let Some(channel) = &node.translation {
            translation = sample_vec3(channel.interpolation, &channel.times, &channel.values, time);
        }
        if let Some(channel) = &node.rotation {
            rotation = sample_quat(channel.interpolation, &channel.times, &channel.values, time);
        }
        if let Some(channel) = &node.scale {
            scale = sample_vec3(channel.interpolation, &channel.times, &channel.values, time);
        }

        Mat4::from_scale_rotation_translation(scale, rotation, translation)
    }
}

#[derive(Debug, Clone)]
pub struct Joint {
    pub node: NodeId,
    pub inverse_bind: Mat4,
}

#[derive(Debug, Clone)]
pub struct Skin {
    pub joints: Vec<Joint>,
}

impl Skin {
    pub fn new(joints: Vec<Joint>) -> Self {
        Self { joints }
    }
}

#[derive(Debug, Clone)]
pub struct SkinPose {
    pub matrices: Vec<Mat4>,
}

impl SkinPose {
    pub fn new(skeleton: &Skin) -> Self {
        Self {
            matrices: vec![Mat4::IDENTITY; skeleton.joints.len()],
        }
    }

    pub fn update(&mut self, skeleton: &Skin, pose: &Pose) {
        for (index, joint) in skeleton.joints.iter().enumerate() {
            self.matrices[index] = pose.world_transforms[joint.node as usize] * joint.inverse_bind;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pose {
    pub world_transforms: Vec<Mat4>,
}

impl Pose {
    pub fn new(world_transforms: &[Mat4]) -> Self {
        Self {
            world_transforms: world_transforms.into(),
        }
    }

    pub fn update(&mut self, asset: &Asset, animation: &AnimationClip, mut time: f32) {
        // Loop animation if run for longer than duration
        if animation.duration > 0.0 {
            time %= animation.duration;
        }

        for index in asset.node_order.iter().cloned() {
            let node = &asset.nodes[index as usize];
            let local_transform = animation.sample_node(index as u32, node.local_transform, time);
            self.world_transforms[index as usize] = match node.parent {
                Some(parent) => self.world_transforms[parent as usize] * local_transform,
                None => local_transform,
            };
        }
    }
}
