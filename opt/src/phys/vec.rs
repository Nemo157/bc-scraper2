use noisy_float::types::R32;
use num_traits::Float;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::cmp::{PartialOrd, Ordering};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Vec2 {
    pub x: R32,
    pub y: R32,
}

impl PartialOrd for Vec2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.x == other.x && self.y == other.y {
            Some(Ordering::Equal)
        } else if self.x < other.x && self.y < other.y {
            Some(Ordering::Less)
        } else if self.x > other.x && self.y > other.y {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl Vec2 {
    pub fn taxicab(self) -> R32 {
        self.x.abs() + self.y.abs()
    }

    pub fn chebyshev(self) -> R32 {
        self.x.abs().max(self.y.abs())
    }

    pub fn euclid_squared(self) -> R32 {
        self.x.abs() * self.x.abs() + self.y.abs() * self.y.abs()
    }

    pub fn clamp(self, max: f32) -> Self {
        if self.euclid_squared() > (max * max) {
            self.unit() * max
        } else {
            self
        }
    }

    pub fn unit(self) -> Self {
        self / self.euclid_squared().sqrt().raw()
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x: R32::new(x), y: R32::new(y) }
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: &Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Vec2> for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Vec2> for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: &Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<&Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: &Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<Vec2> for &mut Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<&Vec2> for &mut Vec2 {
    fn add_assign(&mut self, rhs: &Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<&Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: &Vec2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<Vec2> for &Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<&Vec2> for &Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: &Vec2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<&Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: &Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<Vec2> for &mut Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<&Vec2> for &mut Vec2 {
    fn sub_assign(&mut self, rhs: &Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<&f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: &f32) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<f32> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<&f32> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: &f32) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl MulAssign<f32> for &mut Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<&f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: &f32) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<f32> for &Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<&f32> for &Vec2 {
    type Output = Vec2;

    fn div(self, rhs: &f32) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl DivAssign<f32> for &mut Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Neg for &Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}
