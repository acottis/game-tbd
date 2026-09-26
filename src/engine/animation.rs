use glam::{Mat4, Quat, Vec3};
use gltf::animation::Interpolation;

use crate::assets::{Asset, NodeId};

// TODO: Perf
fn keyframes(times: &[f32], time: f32) -> (usize, usize, f32) {
    if times.len() <= 1 || time <= times[0] {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct LocalTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl LocalTransform {
    #[inline(always)]
    pub fn from_mat4(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

        Self {
            translation,
            rotation,
            scale,
        }
    }

    #[inline(always)]
    pub fn to_mat4(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    #[inline(always)]
    pub fn blend(self, other: Self, weight: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, weight),
            rotation: self.rotation.slerp(other.rotation, weight),
            scale: self.scale.lerp(other.scale, weight),
        }
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
    pub fn sample_node(&self, index: u32, transform: &mut LocalTransform, time: f32) {
        let Some(ref node) = self.nodes[index as usize] else {
            return;
        };

        if let Some(channel) = &node.translation {
            transform.translation =
                sample_vec3(channel.interpolation, &channel.times, &channel.values, time);
        }
        if let Some(channel) = &node.rotation {
            transform.rotation =
                sample_quat(channel.interpolation, &channel.times, &channel.values, time);
        }
        if let Some(channel) = &node.scale {
            transform.scale =
                sample_vec3(channel.interpolation, &channel.times, &channel.values, time);
        }
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
    /// What gets send to the GPU
    pub world_transforms: Vec<Mat4>,
    /// Our current
    pub local_transforms: Vec<LocalTransform>,
}

impl Pose {
    pub fn new(asset: &Asset) -> Self {
        let local_transforms = asset
            .nodes
            .iter()
            .map(|node| node.local_transform)
            .collect();
        Self {
            world_transforms: vec![Mat4::IDENTITY; asset.nodes.len()],
            local_transforms,
        }
    }

    pub fn blend_masked(&mut self, other: &Pose, mask: &BoneMask, weight: f32) {
        debug_assert_eq!(self.local_transforms.len(), other.local_transforms.len());

        debug_assert_eq!(self.local_transforms.len(), mask.weights.len());

        for ((a, b), &mask_weight) in self
            .local_transforms
            .iter_mut()
            .zip(&other.local_transforms)
            .zip(&mask.weights)
        {
            let weight = weight * mask_weight;

            if weight > 0.0 {
                *a = (*a).blend(*b, weight);
            }
        }
    }

    pub fn blend_into(&mut self, other: &Pose, weight: f32) {
        debug_assert_eq!(self.local_transforms.len(), other.local_transforms.len(),);

        for (a, b) in self
            .local_transforms
            .iter_mut()
            .zip(&other.local_transforms)
        {
            *a = (*a).blend(*b, weight);
        }
    }
    pub fn update(&mut self, asset: &Asset) {
        for &index in asset.node_order.iter() {
            let node = &asset.nodes[index as usize];

            let local = self.local_transforms[index as usize].to_mat4();

            self.world_transforms[index as usize] = match node.parent {
                Some(parent) => self.world_transforms[parent as usize] * local,
                None => local,
            };
        }
    }

    pub fn sample(&mut self, asset: &Asset, animation: &AnimationClip, time: f32) {
        for &node_id in &asset.node_order {
            let index = node_id as usize;
            let node = &asset.nodes[index];
            self.local_transforms[index] = node.local_transform;
            animation.sample_node(index as u32, &mut self.local_transforms[index], time);
        }
    }

    pub fn reset_to_rest(&mut self, asset: &Asset) {
        for (pose, node) in self.local_transforms.iter_mut().zip(&asset.nodes) {
            *pose = node.local_transform;
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoneMask {
    weights: Vec<f32>,
}

impl BoneMask {
    pub fn new(asset: &Asset) -> Self {
        Self {
            weights: vec![0.0; asset.nodes.len()],
        }
    }

    pub fn set(&mut self, node: NodeId, weight: f32) {
        self.weights[node as usize] = weight;
    }
}
