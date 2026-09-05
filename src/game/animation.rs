use glam::{Quat, Vec3};
use gltf::animation::{Interpolation, Property};

#[derive(Debug, Clone)]
pub struct AnimationChannel {
    pub property: Property,
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    pub values: AnimationValues,
}

#[derive(Debug, Clone)]
pub enum AnimationValues {
    Translation(Vec<Vec3>),
    Rotation(Vec<Quat>),
    Scale(Vec<Vec3>),
}

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

fn sample_vec3(times: &[f32], values: &[Vec3], time: f32) -> Vec3 {
    let (i0, i1, t) = keyframes(times, time);
    values[i0].lerp(values[i1], t)
}

fn sample_quat(times: &[f32], values: &[Quat], time: f32) -> Quat {
    let (i0, i1, t) = keyframes(times, time);
    values[i0].slerp(values[i1], t)
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub channels: Vec<AnimationChannel>,
    pub duration: f32,
}

impl AnimationClip {
    pub fn sample(&self, mut time: f32) -> (Vec3, Quat, Vec3) {
        // Loop animation if run for longer than duration
        time = time % self.duration;

        let mut translation = Vec3::ZERO;
        let mut rotation = Quat::IDENTITY;
        let mut scale = Vec3::ONE;

        for channel in &self.channels {
            match &channel.values {
                AnimationValues::Translation(values) => {
                    translation = sample_vec3(&channel.times, values, time);
                }

                AnimationValues::Rotation(values) => {
                    rotation = sample_quat(&channel.times, values, time);
                }

                AnimationValues::Scale(values) => {
                    scale = sample_vec3(&channel.times, values, time);
                }
            }
        }
        (translation, rotation, scale)
    }
}
