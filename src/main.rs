extern crate image;
use std::ops::{Add, Sub};


struct Vox {
    x: f32,
    y: f32,
    z: f32
}

impl Vox {
    fn new(v: (f32, f32, f32)) -> Self {
        Self {x: v.0, y:v.1, z:v.2}
    }
}

impl Sub for &Vox {
    type Output = Vox;

    fn sub(self, other: Self) -> Self::Output {
        Self::Output {x: self.x - other.x, y: self.y - other.y, z: self.z - other.z}
    }
}

impl Add for &Vox {
    type Output = Vox;

    fn add(self, other: Self) -> Self::Output {
        Self::Output {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
    }
}


struct Sphere {
    center: Vox,
    radius: f32
}

impl Sphere {

    fn ray_intersect(&self, orig: &Vox, direction: &Vox, t: f32) -> bool {

        let L = &self.center - orig;
        
        todo!()
    }

}


fn render(imgx: u32, imgy: u32) {
    
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let r = (0.3 * x as f32) as u8;
            let b = (0.3 * y as f32) as u8;
            *pixel = image::Rgb([r, 0, b]);
        }
    
    imgbuf.save("test.png");

}

fn main() {

    render(1024, 768);

}
