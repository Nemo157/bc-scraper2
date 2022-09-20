use std::{
    ops::{Add, AddAssign, Div, Mul, Neg},
    time::Duration,
};

use super::{Vec2, Velocity};

#[derive(Debug, Copy, Clone, Default)]
pub struct Acceleration(pub Vec2);

impl Acceleration {
    pub fn new(x: f32, y: f32) -> Self {
        Self::from((x, y))
    }
}

impl From<Vec2> for Acceleration {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<(f32, f32)> for Acceleration {
    fn from((x, y): (f32, f32)) -> Self {
        Self::from(Vec2::from((x, y)))
    }
}

impl Add<Acceleration> for Acceleration {
    type Output = Acceleration;

    fn add(self, rhs: Acceleration) -> Self::Output {
        Acceleration(self.0 + rhs.0)
    }
}

impl Add<&Acceleration> for Acceleration {
    type Output = Acceleration;

    fn add(self, rhs: &Acceleration) -> Self::Output {
        Acceleration(self.0 + rhs.0)
    }
}

impl Add<Acceleration> for &Acceleration {
    type Output = Acceleration;

    fn add(self, rhs: Acceleration) -> Self::Output {
        Acceleration(self.0 + rhs.0)
    }
}

impl Add<&Acceleration> for &Acceleration {
    type Output = Acceleration;

    fn add(self, rhs: &Acceleration) -> Self::Output {
        Acceleration(self.0 + rhs.0)
    }
}

impl AddAssign<Acceleration> for Acceleration {
    fn add_assign(&mut self, rhs: Acceleration) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Acceleration> for Acceleration {
    fn add_assign(&mut self, rhs: &Acceleration) {
        self.0 += rhs.0;
    }
}

impl AddAssign<Acceleration> for &mut Acceleration {
    fn add_assign(&mut self, rhs: Acceleration) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Acceleration> for &mut Acceleration {
    fn add_assign(&mut self, rhs: &Acceleration) {
        self.0 += rhs.0;
    }
}

impl Mul<Duration> for Acceleration {
    type Output = Velocity;

    fn mul(self, rhs: Duration) -> Self::Output {
        Velocity::from(self.0 * rhs.as_secs_f32())
    }
}

impl Mul<&Duration> for Acceleration {
    type Output = Velocity;

    fn mul(self, rhs: &Duration) -> Self::Output {
        Velocity::from(self.0 * rhs.as_secs_f32())
    }
}

impl Mul<Duration> for &Acceleration {
    type Output = Velocity;

    fn mul(self, rhs: Duration) -> Self::Output {
        Velocity::from(self.0 * rhs.as_secs_f32())
    }
}

impl Mul<&Duration> for &Acceleration {
    type Output = Velocity;

    fn mul(self, rhs: &Duration) -> Self::Output {
        Velocity::from(self.0 * rhs.as_secs_f32())
    }
}

impl Div<f32> for Acceleration {
    type Output = Acceleration;

    fn div(self, rhs: f32) -> Self::Output {
        Acceleration::from(self.0 / rhs)
    }
}

impl Div<&f32> for Acceleration {
    type Output = Acceleration;

    fn div(self, rhs: &f32) -> Self::Output {
        Acceleration::from(self.0 / rhs)
    }
}

impl Div<f32> for &Acceleration {
    type Output = Acceleration;

    fn div(self, rhs: f32) -> Self::Output {
        Acceleration::from(self.0 / rhs)
    }
}

impl Div<&f32> for &Acceleration {
    type Output = Acceleration;

    fn div(self, rhs: &f32) -> Self::Output {
        Acceleration::from(self.0 / rhs)
    }
}

impl Neg for Acceleration {
    type Output = Acceleration;

    fn neg(self) -> Self::Output {
        Acceleration(-self.0)
    }
}

impl Neg for &Acceleration {
    type Output = Acceleration;

    fn neg(self) -> Self::Output {
        Acceleration(-self.0)
    }
}
