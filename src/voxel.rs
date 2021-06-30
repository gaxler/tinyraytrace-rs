//! Voxels are 3-D vectors. This is a helper module to 
use std::ops::{Add, Div, Mul, Sub};

/// 3-D vector, this struct includes functions for conveniently perform
#[derive(Clone, Copy, Debug)]
pub struct Vox {
    x: f32,
    y: f32,
    z: f32,
}

impl Vox {
    /// Create an origin vector (0, 0, 0)
    pub fn orig() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }

    /// Create a new vector by specifing its coordinates
    pub fn new(v: (f32, f32, f32)) -> Self {
        Self {
            x: v.0,
            y: v.1,
            z: v.2,
        }
    }

    /// Get the [L2 norm](https://mathworld.wolfram.com/L2-Norm.html) of the vector.
    /// L_2 norm is the lenght of the vector, in 3-D space is basicly the distance of a vector from the origin.
    /// Let say you have 2 vectors v1 and v2, running (v1-v2).l2() will give you the distance between those points.
    /// That is, the distance berween v_1 and v_2 is the length of a vector from v_1 to v_2
    pub fn l2(&self) -> f32 {
        (self.x.powf(2.) + self.y.powf(2.) + self.z.powf(2.)).sqrt()
    }
    /// This gives us the [Dot Product](https://mathworld.wolfram.com/DotProduct.html) of 2 vectors.
    /// This is a very useful quantity for projection of vectors.
    /// Key properאy of the dot-product is this: ![](https://mathworld.wolfram.com/images/equations/DotProduct/NumberedEquation1.gif)
    /// What this means is that the dot-product of two vectors is a product of their lengths and the cosine of the angle between them. [Vector Projection](https://en.wikipedia.org/wiki/Vector_projection) 
    ///
    /// ![](https://upload.wikimedia.org/wikipedia/commons/thumb/9/98/Projection_and_rejection.png/300px-Projection_and_rejection.png)
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Get 1-norm vector
    pub fn normalized(&self) -> Self {
        let d = self.l2();
        Self {
            x: self.x / d,
            y: self.y / d,
            z: self.z / d,
        }
    }

    /// Walk in the direction of the vector
    pub fn walk_dir(&self, v: f32) -> Self {
        Self {
            x: self.x * v,
            y: self.y * v,
            z: self.z * v,
        }
    }
}

impl Sub for Vox {
    type Output = Vox;

    fn sub(self, other: Self) -> Vox {
        Vox {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Add for Vox {
    type Output = Vox;

    fn add(self, other: Self) -> Vox {
        Vox {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
