mod voxel;
extern crate image;

use voxel::Vox;
use std::f32::consts::FRAC_2_PI;

struct Sphere {
    center: Vox,
    radius: f32,
}

impl Sphere {
    
    fn ray_intersect(&self, orig: Vox, direction: Vox) -> bool {
        let v = self.center - orig;
        let u = direction.normalized();
        let dist_orig_proj = v.dot(&u);
        if dist_orig_proj < 0. {
            return false;
        }
        let proj = orig + u.walk_dir(dist_orig_proj);

        let dist_ctr_proj = (self.center - proj).l2();

        if dist_ctr_proj < self.radius {
            return false;
        }

        true

        // let dist_inter_proj = (self.radius.powf(2.) - dist_ctr_proj.powf(2.)).sqrt();
    }
}

fn cast_ray(orig: Vox, dir: Vox, spehre: &Sphere) -> image::Rgb<u8> {
    let dist = f32::MAX;
    if !spehre.ray_intersect(orig, dir) {
        image::Rgb([55, 180, 210])
    } else {
        image::Rgb([125, 125, 80])
    }
    
}

fn render(spehre: &Sphere) {

    let imgx = 1024;
    let imgy = 768;
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    let width = imgx as f32;
    let height = imgy as f32;
    let wh_ratio = width / height;
    let tan_fov = FRAC_2_PI.tan();

    // Iterate over the coordinates and pixels of the image
    for (i, j, pixel) in imgbuf.enumerate_pixels_mut() {

        let rel_w = (i as f32 + 0.5)/ width;
        let rel_h = (j as f32 + 0.5)/ height;

        let x = (2.0*rel_w - 1.0)*tan_fov*wh_ratio;
        let y = (2.0*rel_h - 1.0)*tan_fov;

        let dir = Vox::new((x,y, -1.0)).normalized();

        *pixel = cast_ray(Vox::new((0.,0.,0.)), dir, spehre);
    }

    imgbuf.save("test.png");
}

fn main() {
    let s = Sphere{center: Vox::new((-3., 0., -16.)), radius: 2.0};
    render(&s);
}
