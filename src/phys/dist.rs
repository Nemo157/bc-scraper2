use std::ops::{Add, AddAssign};

use super::Vec2;

#[derive(Debug, Copy, Clone, Default)]
pub struct Distance(pub Vec2);

impl Distance {
    pub fn new(x: f32, y: f32) -> Self {
        Self::from((x, y))
    }

    pub fn taxicab(self) -> f32 {
        self.0.taxicab()
    }

    pub fn chebyshev(self) -> f32 {
        self.0.chebyshev()
    }
}

impl From<Vec2> for Distance {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<(f32, f32)> for Distance {
    fn from((x, y): (f32, f32)) -> Self {
        Self::from(Vec2::from((x, y)))
    }
}

impl From<[f32; 2]> for Distance {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::from(Vec2::from((x, y)))
    }
}

impl Add<Distance> for Distance {
    type Output = Distance;

    fn add(self, rhs: Distance) -> Self::Output {
        Distance(self.0 + rhs.0)
    }
}

impl Add<&Distance> for Distance {
    type Output = Distance;

    fn add(self, rhs: &Distance) -> Self::Output {
        Distance(self.0 + rhs.0)
    }
}

impl Add<Distance> for &Distance {
    type Output = Distance;

    fn add(self, rhs: Distance) -> Self::Output {
        Distance(self.0 + rhs.0)
    }
}

impl Add<&Distance> for &Distance {
    type Output = Distance;

    fn add(self, rhs: &Distance) -> Self::Output {
        Distance(self.0 + rhs.0)
    }
}

impl AddAssign<Distance> for Distance {
    fn add_assign(&mut self, rhs: Distance) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Distance> for Distance {
    fn add_assign(&mut self, rhs: &Distance) {
        self.0 += rhs.0;
    }
}

impl AddAssign<Distance> for &mut Distance {
    fn add_assign(&mut self, rhs: Distance) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Distance> for &mut Distance {
    fn add_assign(&mut self, rhs: &Distance) {
        self.0 += rhs.0;
    }
}
