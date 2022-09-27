use std::ops::{Add, AddAssign, Sub};

use super::{Distance, Vec2};

#[derive(Debug, Copy, Clone, Default, PartialOrd, PartialEq, Eq)]
pub struct Position(pub Vec2);

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self::from((x, y))
    }
}

impl From<Position> for mint::Point2<f32> {
    fn from(pos: Position) -> Self {
        [pos.0.x.raw(), pos.0.y.raw()].into()
    }
}

impl From<&Position> for mint::Point2<f32> {
    fn from(pos: &Position) -> Self {
        [pos.0.x.raw(), pos.0.y.raw()].into()
    }
}

impl From<mint::Point2<f32>> for Position {
    fn from(pos: mint::Point2<f32>) -> Self {
        (pos.x, pos.y).into()
    }
}

impl From<Vec2> for Position {
    fn from(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<(f32, f32)> for Position {
    fn from((x, y): (f32, f32)) -> Self {
        Self::from(Vec2::from((x, y)))
    }
}

impl From<[f32; 2]> for Position {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::from(Vec2::from((x, y)))
    }
}

impl Add<Distance> for Position {
    type Output = Position;

    fn add(self, rhs: Distance) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Add<&Distance> for Position {
    type Output = Position;

    fn add(self, rhs: &Distance) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Add<Distance> for &Position {
    type Output = Position;

    fn add(self, rhs: Distance) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Add<&Distance> for &Position {
    type Output = Position;

    fn add(self, rhs: &Distance) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Sub<Distance> for Position {
    type Output = Position;

    fn sub(self, rhs: Distance) -> Self::Output {
        Position(self.0 - rhs.0)
    }
}

impl AddAssign<Distance> for Position {
    fn add_assign(&mut self, rhs: Distance) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Distance> for Position {
    fn add_assign(&mut self, rhs: &Distance) {
        self.0 += rhs.0;
    }
}

impl AddAssign<Distance> for &mut Position {
    fn add_assign(&mut self, rhs: Distance) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Distance> for &mut Position {
    fn add_assign(&mut self, rhs: &Distance) {
        self.0 += rhs.0;
    }
}

impl Sub<Position> for Position {
    type Output = Distance;

    fn sub(self, rhs: Position) -> Self::Output {
        Distance::from(self.0 - rhs.0)
    }
}

impl Sub<&Position> for Position {
    type Output = Distance;

    fn sub(self, rhs: &Position) -> Self::Output {
        Distance::from(self.0 - rhs.0)
    }
}

impl Sub<Position> for &Position {
    type Output = Distance;

    fn sub(self, rhs: Position) -> Self::Output {
        Distance::from(self.0 - rhs.0)
    }
}

impl Sub<&Position> for &Position {
    type Output = Distance;

    fn sub(self, rhs: &Position) -> Self::Output {
        Distance::from(self.0 - rhs.0)
    }
}
