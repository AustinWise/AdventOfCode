use std::ops::Add;
use std::ops::Sub;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Vec2 { x, y }
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn all() -> [Self; 4] {
        [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ]
    }

    pub fn move_vector(self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0, -1),
            Direction::Down => Vec2::new(0, 1),
            Direction::Left => Vec2::new(-1, 0),
            Direction::Right => Vec2::new(1, 0),
        }
    }

    pub fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    pub fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let x = Vec2::new(1, 3) + Vec2::new(5, 7);
        assert_eq!(x, Vec2::new(6, 10));
    }

    #[test]
    fn test_sub() {
        let x = Vec2::new(1, 3) - Vec2::new(5, 2);
        assert_eq!(x, Vec2::new(-4, 1));
    }
}
