use glam::{Quat, Vec3};
use gltf::animation::Interpolation;

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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct AnimationClip {
    translations: Vec<Translation>,
    rotations: Vec<Rotation>,
    scales: Vec<Scale>,

    duration: f32,
}

impl AnimationClip {
    pub fn new(
        translations: Vec<Translation>,
        rotations: Vec<Rotation>,
        scales: Vec<Scale>,
        duration: f32,
    ) -> Self {
        Self {
            translations,
            rotations,
            scales,
            duration,
        }
    }
}

impl AnimationClip {
    pub fn sample(&self, mut time: f32) -> (Vec3, Quat, Vec3) {
        // Loop animation if run for longer than duration
        time = time % self.duration;

        let mut translation = Vec3::ZERO;
        let mut rotation = Quat::IDENTITY;
        let mut scale = Vec3::ONE;

        for channel in &self.translations {
            translation = sample_vec3(channel.interpolation, &channel.times, &channel.values, time);
        }
        for channel in &self.rotations {
            rotation = sample_quat(channel.interpolation, &channel.times, &channel.values, time);
        }
        for channel in &self.scales {
            scale = sample_vec3(channel.interpolation, &channel.times, &channel.values, time);
        }

        (translation, rotation, scale)
    }
}
