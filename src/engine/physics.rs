use std::collections::HashMap;

use glam::{Mat3, Mat4, Vec3};

use crate::assets::Asset;

pub const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn empty() -> Self {
        let min = Vec3::splat(f32::INFINITY);
        let max = Vec3::splat(f32::NEG_INFINITY);
        Self { min, max }
    }

    pub fn union(&self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
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
        // We have not reached the collision yet
        let collision_is_after_movement_ends = collision_time > 1.0;
        // If we are inside the collision before we start moving it has
        // "already happened"
        let collision_ended_before_movement_starts = collision_time < 0.0;

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

#[derive(Eq, Hash, PartialEq)]
struct CellCoord {
    x: i32,
    z: i32,
}

impl CellCoord {
    const SIZE: f32 = 8.0;

    fn from_position(position: Vec3) -> Self {
        Self {
            x: (position.x / Self::SIZE).floor() as i32,
            z: (position.z / Self::SIZE).floor() as i32,
        }
    }
}

pub struct GroundCollision {
    triangles: Vec<[Vec3; 3]>,
    grid: HashMap<CellCoord, Vec<usize>>,
}

impl GroundCollision {
    pub fn new(model: &Asset, transform: Mat4) -> Self {
        let mut triangles = Vec::new();

        for render_node in &model.render_nodes {
            let transform = transform * model.rest_world_transforms[render_node.node as usize];

            let mesh = &model.meshes[render_node.mesh as usize];
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

        let mut grid: HashMap<CellCoord, Vec<usize>> = HashMap::new();

        for (index, triangle) in triangles.iter().enumerate() {
            let min_x = triangle[0].x.min(triangle[1].x).min(triangle[2].x);
            let max_x = triangle[0].x.max(triangle[1].x).max(triangle[2].x);
            let min_z = triangle[0].z.min(triangle[1].z).min(triangle[2].z);
            let max_z = triangle[0].z.max(triangle[1].z).max(triangle[2].z);

            let min_cell = CellCoord::from_position(Vec3::new(min_x, 0.0, min_z));
            let max_cell = CellCoord::from_position(Vec3::new(max_x, 0.0, max_z));

            for z in min_cell.z..=max_cell.z {
                for x in min_cell.x..=max_cell.x {
                    grid.entry(CellCoord { x, z }).or_default().push(index);
                }
            }
        }

        Self { triangles, grid }
    }

    pub fn height_at(&self, point: Vec3) -> Option<f32> {
        let cell = CellCoord::from_position(point);

        let triangles = self.grid.get(&cell)?;

        triangles
            .iter()
            .filter_map(|&index| triangle_height(self.triangles[index], point))
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
