//! Column Major

use bytemuck::{Pod, Zeroable};

#[derive(Debug)]
pub struct Mat3 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

impl Mat3 {
    pub fn rotation_x(theta: f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        Self {
            x: Vec3::new(1.0, 0.0, 0.0),
            y: Vec3::new(0.0, cos, -sin),
            z: Vec3::new(0.0, sin, cos),
        }
    }
    pub fn rotation_y(theta: f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        Self {
            x: Vec3::new(cos, 0.0, sin),
            y: Vec3::new(0.0, 1.0, 0.0),
            z: Vec3::new(-sin, 0.0, cos),
        }
    }
    pub fn rotation_z(theta: f32) -> Self {
        let cos = theta.cos();
        let sin = theta.sin();
        Self {
            x: Vec3::new(cos, -sin, 0.0),
            y: Vec3::new(sin, cos, 0.0),
            z: Vec3::new(0.0, 0.0, 1.0),
        }
    }
}

#[derive(Pod, Zeroable, Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Mat4 {
    #[inline(always)]
    pub const fn identity() -> Self {
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
    #[inline(always)]
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(translation.x, translation.y, translation.z, 1.0),
        }
    }
    pub const fn from_scaling(scale: Vec3) -> Self {
        Self {
            x: Vec4::new(scale.x, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, scale.y, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, scale.z, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        let q = rotation.normalise();

        let x2 = q.x + q.x;
        let y2 = q.y + q.y;
        let z2 = q.z + q.z;

        let xx = q.x * x2;
        let xy = q.x * y2;
        let xz = q.x * z2;

        let yy = q.y * y2;
        let yz = q.y * z2;

        let zz = q.z * z2;

        let wx = q.w * x2;
        let wy = q.w * y2;
        let wz = q.w * z2;

        Self {
            x: Vec4::new(1.0 - (yy + zz), xy + wz, xz - wy, 0.0),
            y: Vec4::new(xy - wz, 1.0 - (xx + zz), yz + wx, 0.0),
            z: Vec4::new(xz + wy, yz - wx, 1.0 - (xx + yy), 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn from_scale_rotation_translation(scale: Vec3, rotation: Quat, translation: Vec3) -> Self {
        let mut result = Self::from_rotation(rotation);

        result.x.x *= scale.x;
        result.x.y *= scale.x;
        result.x.z *= scale.x;

        result.y.x *= scale.y;
        result.y.y *= scale.y;
        result.y.z *= scale.y;

        result.z.x *= scale.z;
        result.z.y *= scale.z;
        result.z.z *= scale.z;

        result.w = Vec4::new(translation.x, translation.y, translation.z, 1.0);

        result
    }
}

impl std::ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;

    #[inline(always)]
    fn mul(self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.x.x * v.x + self.y.x * v.y + self.z.x * v.z + self.w.x * v.w,
            self.x.y * v.x + self.y.y * v.y + self.z.y * v.z + self.w.y * v.w,
            self.x.z * v.x + self.y.z * v.y + self.z.z * v.z + self.w.z * v.w,
            self.x.w * v.x + self.y.w * v.y + self.z.w * v.z + self.w.w * v.w,
        )
    }
}

impl std::ops::Mul<Mat4> for Mat4 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Mat4) -> Self {
        Self {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
            w: self * rhs.w,
        }
    }
}
#[derive(Pod, Zeroable, Copy, Clone, Default, Debug, PartialEq)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub const fn zeroes() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    pub const fn xyz(xyz: f32) -> Self {
        Self::new(xyz, xyz, xyz)
    }
    pub const fn y() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }
    pub const fn x() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }
    pub const fn dot(&self, rhs: &Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }
    pub const fn cross(&self, rhs: &Self) -> Self {
        Vec3::new(
            (self.y * rhs.z) - (self.z * rhs.y),
            (self.z * rhs.x) - (self.x * rhs.z),
            (self.x * rhs.y) - (self.y * rhs.x),
        )
    }
    pub fn len(&self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalise(&self) -> Self {
        let len = self.len();
        if len == 0.0 {
            return Vec3::zeroes();
        }
        self / len
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(value: [f32; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl core::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl core::ops::Div<f32> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl core::ops::Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x.x * rhs.x + self.y.x * rhs.y + self.z.x * rhs.z,
            y: self.x.y * rhs.x + self.y.y * rhs.y + self.z.y * rhs.z,
            z: self.x.z * rhs.x + self.y.z * rhs.y + self.z.z * rhs.z,
        }
    }
}

impl core::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl core::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl core::ops::Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl core::ops::Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl core::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl core::ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

#[derive(Pod, Zeroable, Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub const fn identity() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn normalise(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();

        if len == 0.0 {
            return Self::identity();
        }

        Self::new(self.x / len, self.y / len, self.z / len, self.w / len)
    }

    pub fn slerp(self, mut rhs: Self, t: f32) -> Self {
        let mut dot = self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w;

        if dot < 0.0 {
            rhs = Self::new(-rhs.x, -rhs.y, -rhs.z, -rhs.w);

            dot = -dot;
        }

        if dot > 0.9995 {
            return Self::new(
                self.x + t * (rhs.x - self.x),
                self.y + t * (rhs.y - self.y),
                self.z + t * (rhs.z - self.z),
                self.w + t * (rhs.w - self.w),
            )
            .normalise();
        }

        let theta = dot.acos();
        let sin_theta = theta.sin();

        let a = ((1.0 - t) * theta).sin() / sin_theta;
        let b = (t * theta).sin() / sin_theta;

        Self::new(
            self.x * a + rhs.x * b,
            self.y * a + rhs.y * b,
            self.z * a + rhs.z * b,
            self.w * a + rhs.w * b,
        )
    }
}

impl From<[f32; 4]> for Quat {
    fn from(value: [f32; 4]) -> Self {
        Self::new(value[0], value[1], value[2], value[3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mat4_identity_multiplication() {
        let mat_a = Mat4 {
            x: Vec4::new(1.0, 2.0, 3.0, 4.0),
            y: Vec4::new(5.0, 6.0, 7.0, 8.0),
            z: Vec4::new(9.0, 10.0, 11.0, 12.0),
            w: Vec4::new(13.0, 14.0, 15.0, 16.0),
        };

        let mat_b = Mat4::identity();

        let result = mat_a * mat_b;

        let expected = Mat4 {
            x: Vec4::new(1.0, 2.0, 3.0, 4.0),
            y: Vec4::new(5.0, 6.0, 7.0, 8.0),
            z: Vec4::new(9.0, 10.0, 11.0, 12.0),
            w: Vec4::new(13.0, 14.0, 15.0, 16.0),
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_eq() {
        let a = Mat4::identity();
        let b = Mat4::identity();

        assert_eq!(a, b);
    }
    #[test]
    fn test_ne() {
        let mut a = Mat4::identity();
        let b = Mat4::identity();
        a.x.w = 0.5;

        assert_ne!(a, b);
    }

    #[test]
    fn test_col_major_matrix_multiplication() {
        let mat_a = Mat4 {
            x: Vec4::new(1.0, 2.0, 3.0, 4.0),
            y: Vec4::new(5.0, 6.0, 7.0, 8.0),
            z: Vec4::new(9.0, 10.0, 11.0, 12.0),
            w: Vec4::new(13.0, 14.0, 15.0, 16.0),
        };

        let mat_b = Mat4 {
            x: Vec4::new(2.0, 0.0, 1.0, 3.0),
            y: Vec4::new(1.0, -1.0, 0.0, 2.0),
            z: Vec4::new(0.0, 3.0, 2.0, 1.0),
            w: Vec4::new(1.0, 1.0, 0.0, 0.0),
        };

        let result = mat_a * mat_b;

        let expected = Mat4 {
            x: Vec4::new(50.0, 56.0, 62.0, 68.0),
            y: Vec4::new(22.0, 24.0, 26.0, 28.0),
            z: Vec4::new(46.0, 52.0, 58.0, 64.0),
            w: Vec4::new(6.0, 8.0, 10.0, 12.0),
        };

        assert_eq!(result, expected);
    }
}
