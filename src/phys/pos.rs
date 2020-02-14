use std::ops::{Add, AddAssign};

use super::Vec2;

#[derive(Debug, Copy, Clone, Default)]
pub struct Position(pub Vec2);

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self::from((x, y))
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

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Add<&Position> for Position {
    type Output = Position;

    fn add(self, rhs: &Position) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Add<Position> for &Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl Add<&Position> for &Position {
    type Output = Position;

    fn add(self, rhs: &Position) -> Self::Output {
        Position(self.0 + rhs.0)
    }
}

impl AddAssign<Position> for Position {
    fn add_assign(&mut self, rhs: Position) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Position> for Position {
    fn add_assign(&mut self, rhs: &Position) {
        self.0 += rhs.0;
    }
}

impl AddAssign<Position> for &mut Position {
    fn add_assign(&mut self, rhs: Position) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&Position> for &mut Position {
    fn add_assign(&mut self, rhs: &Position) {
        self.0 += rhs.0;
    }
}
