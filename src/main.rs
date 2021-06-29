mod voxel;
extern crate image;

use std::f32::consts::FRAC_2_PI;
use voxel::Vox;

struct Sphere {
    center: Vox,
    radius: f32,
    material: Material
}

#[derive(Clone, Copy)]
struct Material {
    color: (f32, f32, f32),
    pixel: image::Rgb<u8>
}

impl Material {

    fn new(color: (f32, f32, f32)) -> Self {
        let (r, g, b) = color;
        let pixel = image::Rgb([(255.*r) as u8, (255.*g) as u8, (255.*b) as u8]);
        Self{color, pixel}
    }

}

impl Default for Material {
    fn default() -> Self {
        Self::new( (0.2, 0.7, 0.8))
    }
}
/// Point where our ray hits and object.
enum HitPoint {
    Point(Vox),
    None,
}

impl Sphere {
    fn ray_intersect(&self, orig: Vox, direction: Vox) -> HitPoint {
        let v = self.center - orig;
        let u = direction.normalized();

        let dist_orig_proj = v.dot(&u);

        if dist_orig_proj < 0. {
            return HitPoint::None;
        }
        let proj = orig + u.walk_dir(dist_orig_proj);

        let dist_ctr_proj = (self.center - proj).l2();

        if dist_ctr_proj > self.radius {
            return HitPoint::None;
        }

        let dist_proj_intersect1 = (self.radius.powf(2.0) - dist_ctr_proj.powf(2.)).sqrt();

        match (
            dist_orig_proj - dist_proj_intersect1,
            dist_orig_proj + dist_proj_intersect1,
        ) {
            (o_i1, _) if o_i1 > 0. => HitPoint::Point(proj.walk_dir(o_i1)),
            // Origin is inside the sphere
            // Assuming light can move thorugh sphere we'll see the other intersection point
            (_, o_i2) if o_i2 > 0. => HitPoint::Point(proj.walk_dir(o_i2)), 
            _ => HitPoint::None,
        }

    }
}

fn cast_ray(orig: Vox, dir: Vox, scene: &[Sphere]) -> image::Rgb<u8> {
    let mut dist = f32::MAX;
    let mut res = image::Rgb([55, 180, 210]);
    let mut normal = Vox::orig();
    //let material;

    for s in scene.iter() {
        match s.ray_intersect(orig, dir) {
            NearestIntersection::Point(p) if (p-orig).l2() < dist => {
                dist = (p-orig).l2();
                res = s.material;
                // something with material
            }
            _ => continue
        }
    }

    res
}

fn render(spehres: Vec<Sphere>) {
    let imgx = 1024;
    let imgy = 768;
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    let width = imgx as f32;
    let height = imgy as f32;
    let wh_ratio = width / height;
    let tan_fov = FRAC_2_PI.tan();

    let ray_origin = Vox::orig();

    // Iterate over the coordinates and pixels of the image
    for (i, j, pixel) in imgbuf.enumerate_pixels_mut() {
        let rel_w = (i as f32 + 0.5) / width;
        let rel_h = (j as f32 + 0.5) / height;

        let x = (2.0 * rel_w - 1.0) * tan_fov * wh_ratio;
        let y = -(2.0 * rel_h - 1.0) * tan_fov;

        let dir = Vox::new((x, y, -1.0)).normalized();

        *pixel = cast_ray(ray_origin, dir, &spehres).pixel;
    }

    imgbuf.save("test.png");
}

fn main() {

    let ivory = Material::new((0.4, 0.4, 0.3));
    let red_rubber = Material::new((0.3, 0.1, 0.1));

    let s = Sphere {
        center: Vox::new((-3., 0., -16.)),
        radius: 2.0,
        material: ivory
    };

    let s2 = Sphere {
        center: Vox::new((-1., -1.5, -12.)),
        radius: 2.0,
        material: red_rubber
    };

    let s3 = Sphere {
        center: Vox::new((1.5, -0.5, -18.)),
        radius: 3.0,
        material: red_rubber
    };

    let s4 = Sphere {
        center: Vox::new((7., 5., -18.)),
        radius: 4.0,
        material: ivory
    };

    
    render(vec![s, s2, s3, s4]);
}
