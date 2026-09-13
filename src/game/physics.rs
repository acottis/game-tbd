use glam::{Mat3, Mat4, Vec3};

use crate::assets::AssetModel;

pub const GRAVITY: Vec3 = Vec3::new(0.0, -7.0, 0.0);

pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(model: &AssetModel) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        for mesh in &model.meshes {
            for primitive in &mesh.primitives {
                min = min.min(primitive.bounding_box.min.into());
                max = max.max(primitive.bounding_box.max.into());
            }
        }

        Self { min, max }
    }

    pub fn sweep(&self, movement: Vec3, other: &BoundingBox) -> Option<(f32, Vec3)> {
        let mut entry_time = Vec3::splat(f32::NEG_INFINITY);
        let mut exit_time = Vec3::splat(f32::INFINITY);

        for axis in 0..3 {
            let velocity = movement[axis];

            if velocity == 0.0 {
                // If any axis is not colliding, then it is not a collision
                if self.max[axis] <= other.min[axis] || self.min[axis] >= other.max[axis] {
                    return None;
                }

                // No movement on this axis, skip
                continue;
            }
            if velocity > 0.0 {
                entry_time[axis] = (other.min[axis] - self.max[axis]) / velocity;
                exit_time[axis] = (other.max[axis] - self.min[axis]) / velocity;
            } else {
                entry_time[axis] = (other.max[axis] - self.min[axis]) / velocity;
                exit_time[axis] = (other.min[axis] - self.max[axis]) / velocity;
            }
        }

        // When the last axis collided, therefore the time of collision
        let collision_time = entry_time.x.max(entry_time.y).max(entry_time.z);
        // When we are no longer colliding
        let seperation_time = exit_time.x.min(exit_time.y).min(exit_time.z);

        let axes_never_collide_simultaneously = collision_time > seperation_time;
        let collision_is_after_movement_ends = collision_time > 1.0;
        // TODO: Not sure if this is possible
        let collision_ended_before_movement_starts = seperation_time < 0.0;

        if axes_never_collide_simultaneously
            || collision_is_after_movement_ends
            || collision_ended_before_movement_starts
        {
            return None;
        }

        // The highest entry time is _when_ we collided as that is when all 3 axis overlapped
        let normal = if entry_time.x >= entry_time.y && entry_time.x >= entry_time.z {
            // We collided on x
            if movement.x > 0.0 {
                Vec3::NEG_X
            } else {
                Vec3::X
            }
        } else if entry_time.y >= entry_time.z {
            // We collided on y
            if movement.y > 0.0 {
                Vec3::NEG_Y
            } else {
                Vec3::Y
            }
        } else {
            // We collided on z
            if movement.z > 0.0 {
                Vec3::NEG_Z
            } else {
                Vec3::Z
            }
        };

        Some((collision_time.max(0.0), normal))
    }

    pub fn transform(&self, transform: Mat4) -> Self {
        let center = (self.min + self.max) * 0.5;
        let extents = (self.max - self.min) * 0.5;

        let center = transform.transform_point3(center);
        let extents = Mat3::from_mat4(transform).abs() * extents;

        Self {
            min: center - extents,
            max: center + extents,
        }
    }
}

pub struct GroundCollision {
    triangles: Vec<[Vec3; 3]>,
}

impl GroundCollision {
    pub fn new(model: &AssetModel, transform: Mat4) -> Self {
        let mut triangles = Vec::new();

        for mesh in &model.meshes {
            for primitive in &mesh.primitives {
                for indices in primitive.indices.chunks_exact(3) {
                    let a = transform
                        .transform_point3(primitive.vertices[indices[0] as usize].position());
                    let b = transform
                        .transform_point3(primitive.vertices[indices[1] as usize].position());
                    let c = transform
                        .transform_point3(primitive.vertices[indices[2] as usize].position());
                    let normal = (b - a).cross(c - a).normalize();

                    // We only care about walkable collision so we discard
                    // negative y
                    if normal.y > 0.0 {
                        triangles.push([a, b, c]);
                    }
                }
            }
        }

        Self { triangles }
    }

    pub fn height_at(&self, point: Vec3) -> Option<f32> {
        self.triangles
            .iter()
            .filter_map(|triangle| triangle_height(*triangle, point))
            .max_by(f32::total_cmp)
    }
}

fn triangle_height([a, b, c]: [Vec3; 3], point: Vec3) -> Option<f32> {
    let area = cross_xz(b - a, c - a);
    if area.abs() <= f32::EPSILON {
        return None;
    }

    let u = cross_xz(b - point, c - point) / area;
    let v = cross_xz(c - point, a - point) / area;
    let w = 1.0 - u - v;
    if u < 0.0 || v < 0.0 || w < 0.0 {
        return None;
    }

    Some(a.y * u + b.y * v + c.y * w)
}

fn cross_xz(a: Vec3, b: Vec3) -> f32 {
    a.x * b.z - a.z * b.x
}
