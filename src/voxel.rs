/// This is a helper module to create 3d vec functionaliy
use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy)]
pub struct Vox {
    x: f32,
    y: f32,
    z: f32,
}

impl Vox {
    pub fn new(v: (f32, f32, f32)) -> Self {
        Self {
            x: v.0,
            y: v.1,
            z: v.2,
        }
    }

    pub fn l2(&self) -> f32 {
        (self.x.powf(2.) + self.y.powf(2.) + self.z.powf(2.)).sqrt()
    }

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
