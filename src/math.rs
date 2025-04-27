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

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(translation.x, translation.y, translation.z, 1.0),
        }
    }
    pub fn transpose(self) -> Mat4 {
        Mat4 {
            x: Vec4::new(self.x.x, self.y.x, self.z.x, self.w.x),
            y: Vec4::new(self.x.y, self.y.y, self.z.y, self.w.y),
            z: Vec4::new(self.x.z, self.y.z, self.z.z, self.w.z),
            w: Vec4::new(self.x.w, self.y.w, self.z.w, self.w.w),
        }
    }
}

impl std::ops::Mul<Mat4> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Mat4) -> Self::Output {
        let rhs = rhs.transpose();
        Self {
            x: Vec4 {
                x: self.x.dot(rhs.x),
                y: self.x.dot(rhs.y),
                z: self.x.dot(rhs.z),
                w: self.x.dot(rhs.w),
            },
            y: Vec4 {
                x: self.y.dot(rhs.x),
                y: self.y.dot(rhs.y),
                z: self.y.dot(rhs.z),
                w: self.y.dot(rhs.w),
            },
            z: Vec4 {
                x: self.z.dot(rhs.x),
                y: self.z.dot(rhs.y),
                z: self.z.dot(rhs.z),
                w: self.z.dot(rhs.w),
            },
            w: Vec4 {
                x: self.w.dot(rhs.x),
                y: self.w.dot(rhs.y),
                z: self.w.dot(rhs.z),
                w: self.w.dot(rhs.w),
            },
        }
    }
}

#[derive(
    bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Default, Debug, PartialEq,
)]
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
    pub const fn dot(&self, rhs: Self) -> f32 {
        (self.x * rhs.x)
            + (self.y * rhs.y)
            + (self.z * rhs.z)
            + (self.w * rhs.w)
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
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

    fn mul(self, rhs: Vec3) -> Self::Output {
        let x = self.x.x * rhs.x + self.y.x * rhs.y + self.z.x * rhs.z;
        let y = self.x.y * rhs.x + self.y.y * rhs.y + self.z.y * rhs.z;
        let z = self.x.z * rhs.x + self.y.z * rhs.y + self.z.z * rhs.z;

        Vec3 { x, y, z }
    }
}
impl core::ops::Add for Vec3 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mat4_identity_multiplication() {
        // Define two sample matrices
        let mat_a = Mat4 {
            x: Vec4::new(1.0, 2.0, 3.0, 4.0),
            y: Vec4::new(5.0, 6.0, 7.0, 8.0),
            z: Vec4::new(9.0, 10.0, 11.0, 12.0),
            w: Vec4::new(13.0, 14.0, 15.0, 16.0),
        };

        let mat_b = Mat4::identity();

        // Perform matrix multiplication
        let result = mat_a * mat_b;

        let expected = Mat4 {
            x: Vec4::new(1.0, 2.0, 3.0, 4.0),
            y: Vec4::new(5.0, 6.0, 7.0, 8.0),
            z: Vec4::new(9.0, 10.0, 11.0, 12.0),
            w: Vec4::new(13.0, 14.0, 15.0, 16.0),
        };

        // Assert that the result of multiplication is correct
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
    fn test_row_major_matrix_multiplication() {
        let mat_a = Mat4 {
            x: Vec4::new(1.0, 2.0, 3.0, 4.0),
            y: Vec4::new(5.0, 6.0, 7.0, 8.0),
            z: Vec4::new(9.0, 10.0, 11.0, 12.0),
            w: Vec4::new(13.0, 14.0, 15.0, 16.0),
        };

        let mat_b = Mat4 {
            x: Vec4::new(1.0, 0.0, 0.0, 1.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        };

        let result = mat_a * mat_b;

        let expected = Mat4 {
            x: Vec4::new(1.0, 2.0, 3.0, 5.0),
            y: Vec4::new(5.0, 6.0, 7.0, 13.0),
            z: Vec4::new(9.0, 10.0, 11.0, 21.0),
            w: Vec4::new(13.0, 14.0, 15.0, 29.0),
        };

        assert_eq!(result, expected);
    }
}
