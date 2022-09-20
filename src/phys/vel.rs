use std::{
    ops::{Add, AddAssign, Mul, MulAssign},
    time::Duration,
};

use super::{Distance, Vec2};

#[derive(Debug, Copy, Clone, Default)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self::from((x, y))
    }
}

impl From<Vec2> for Velocity {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<(f32, f32)> for Velocity {
    fn from((x, y): (f32, f32)) -> Self {
        Self::from(Vec2::from((x, y)))
    }
}

impl Add<Velocity> for Velocity {
    type Output = Velocity;

    fn add(self, rhs: Velocity) -> Self::Output {
        Velocity(self.0 + rhs.0)
    }
}

impl Add<&Velocity> for Velocity {
    type Output = Velocity;

    fn add(self, rhs: &Velocity) -> Self::Output {
        Velocity(self.0 + rhs.0)
    }
}

impl Add<Velocity> for &Velocity {
    type Output = Velocity;

    fn add(self, rhs: Velocity) -> Self::Output {
        Velocity(self.0 + rhs.0)
    }
}

impl Add<&Velocity> for &Velocity {
    type Output = Velocity;

    fn add(self, rhs: &Velocity) -> Self::Output {
        Velocity(self.0 + rhs.0)
    }
}

impl AddAssign<Velocity> for Velocity {
    fn add_assign(&mut self, rhs: Velocity) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Velocity> for Velocity {
    fn add_assign(&mut self, rhs: &Velocity) {
        self.0 += rhs.0;
    }
}

impl AddAssign<Velocity> for &mut Velocity {
    fn add_assign(&mut self, rhs: Velocity) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Velocity> for &mut Velocity {
    fn add_assign(&mut self, rhs: &Velocity) {
        self.0 += rhs.0;
    }
}

impl Mul<Duration> for Velocity {
    type Output = Distance;

    fn mul(self, rhs: Duration) -> Self::Output {
        Distance::from(self.0 * rhs.as_secs_f32())
    }
}

impl Mul<&Duration> for Velocity {
    type Output = Distance;

    fn mul(self, rhs: &Duration) -> Self::Output {
        Distance::from(self.0 * rhs.as_secs_f32())
    }
}

impl Mul<Duration> for &Velocity {
    type Output = Distance;

    fn mul(self, rhs: Duration) -> Self::Output {
        Distance::from(self.0 * rhs.as_secs_f32())
    }
}

impl Mul<&Duration> for &Velocity {
    type Output = Distance;

    fn mul(self, rhs: &Duration) -> Self::Output {
        Distance::from(self.0 * rhs.as_secs_f32())
    }
}

impl MulAssign<f32> for Velocity {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}

impl MulAssign<f32> for &mut Velocity {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}
