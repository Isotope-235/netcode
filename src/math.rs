//! Mathematical types and utility.

use std::ops::{Add, AddAssign, Mul, Sub};

/// A 2-dimensional vector, containing x and y components.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Vec2 {
    /// The first scalar, or the horizontal direction.
    pub x: f64,
    /// The second scalar, or the vertical direction.
    pub y: f64,
}

impl Vec2 {
    pub const NULL: Self = Vec2::new(0., 0.);

    pub const fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    pub fn len(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn dist(self, rhs: Vec2) -> f64 {
        (self - rhs).len()
    }
}

impl PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Add for Vec2 {
    type Output = Self;
    /// Adds the vector `rhs` to `self`.
    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    /// Subtracts the vector `rhs` from `self`.
    /// In game space, this can be thought of as finding the vector that points from `rhs` to `self`.
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    /// Adds the vector `rhs` to `self` and assigns the resulting value to `self`.
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;
    /// Multiplies the vector `self` with a scalar value.
    fn mul(self, rhs: f64) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul for Vec2 {
    type Output = f64;
    /// Takes the dot product of the two vectors.
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn len() {
        let v = Vec2::new(3.0, 4.0);
        let length = v.len();
        assert_eq!(length, 5.0);
    }

    #[test]
    fn dist() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(4.0, 6.0);
        let distance = v1.dist(v2);
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn eq() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(1.0, 2.0);
        let v3 = Vec2::new(3.0, 4.0);
        assert!(v1 == v2);
        assert!(v1 != v3);
    }

    #[test]
    fn add() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let result = v1 + v2;
        assert_eq!(result, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn sub() {
        let v1 = Vec2::new(5.0, 6.0);
        let v2 = Vec2::new(3.0, 4.0);
        let result = v1 - v2;
        assert_eq!(result, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn add_assign() {
        let mut v = Vec2::new(1.0, 2.0);
        v += Vec2::new(3.0, 4.0);
        assert_eq!(v, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn mul_scalar() {
        let v = Vec2::new(2.0, 3.0);
        let result = v * 2.0;
        assert_eq!(result, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn dot() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let result = v1 * v2;
        assert_eq!(result, 11.0);
    }
}
