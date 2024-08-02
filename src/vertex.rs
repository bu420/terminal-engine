use std::{iter::zip, ops::{Add, AddAssign, Div, Sub}};

use glm::Vec4;

pub fn lerp(a: f32, b: f32, amount: f32) -> f32 {
    return amount * b + (1.0 - amount) * a;
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec4,
    pub attributes: Vec<f32>
}

impl Vertex {
    pub fn lerp(a: &Vertex, b: &Vertex, amount: f32) -> Self {
        debug_assert!(a.attributes.len() == b.attributes.len());

        Self {
            position: a.position.lerp(&b.position, amount),
            attributes: zip(&a.attributes, &b.attributes)
                .map(|(a, b)| lerp(*a, *b, amount)).collect()
        }
    }
}

impl Add<&Vertex> for &Vertex {
    type Output = Vertex;

    fn add(self, rhs: &Vertex) -> Vertex {        
        Vertex {
            position: self.position + rhs.position,
            attributes: zip(&self.attributes, &rhs.attributes).map(|(a, b)| a + b).collect()
        }
    }
}

impl AddAssign<&Vertex> for Vertex {
    fn add_assign(&mut self, rhs: &Vertex) {
        self.position += rhs.position;

        for (a, b) in zip(&mut self.attributes, &rhs.attributes) {
            *a += b;
        }
    }
}

impl Sub<&Vertex> for &Vertex {
    type Output = Vertex;

    fn sub(self, rhs: &Vertex) -> Vertex {        
        Vertex {
            position: self.position - rhs.position,
            attributes: zip(&self.attributes, &rhs.attributes).map(|(a, b)| a - b).collect()
        }
    }
}

impl Div<usize> for &Vertex {
    type Output = Vertex;

    fn div(self, rhs: usize) -> Vertex {        
        Vertex {
            position: self.position / rhs as f32,
            attributes: self.attributes.iter().map(|a| a / rhs as f32).collect()
        }
    }
}