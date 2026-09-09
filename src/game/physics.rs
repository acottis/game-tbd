use glam::{Mat3, Mat4, Vec3};

use crate::assets::AssetModel;

pub const GRAVITY: Vec3 = Vec3::new(0.0, -5.0, 0.0);

pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn new(model: &AssetModel) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        for mesh in &model.meshes {
            min = min.min(mesh.bounding_box.min.into());
            max = max.max(mesh.bounding_box.max.into());
        }

        Self { min, max }
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
            for indices in mesh.indices.chunks_exact(3) {
                let a = transform.transform_point3(mesh.vertices[indices[0] as usize].position());
                let b = transform.transform_point3(mesh.vertices[indices[1] as usize].position());
                let c = transform.transform_point3(mesh.vertices[indices[2] as usize].position());
                let normal = (b - a).cross(c - a).normalize();

                // We only care about walkable collision so we discard
                // negative y
                if normal.y > 0.0 {
                    triangles.push([a, b, c]);
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
