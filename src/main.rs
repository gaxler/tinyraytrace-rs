//! This is a basic ray tracer. It is based on [Understandable RayTracing in 256 lines of bare C++](https://github.com/ssloy/tinyraytracer/wiki)
//! Comming from a computer vision background, I had some idea how ray tracing might work, this is for me to actually understand it.
//! To make it more fun, I wanna try and write documentation that's going to cover some fundamental concepts as part of it.
//!
//! ( _This is a work in progress and I don't add documentation in a linear fashion, so it might be a bit early to try and read it_)
//!
//! ## What is Ray Tracing?
//! (_caveat: I'm learning about it pretty much for the first time as I implement this._)
//!
//!
//! We have a 3-D model of some object in the world, and we want to epxress this model as a 2-D computre image. To construct our image we need to decide what color to assign to each pixel of the image.
//! Since the real world is complex we don't have an easy method to determine the color we should assign to each image pixel and we relay on simulations of light rays.
//! Now, lets say more about what we mean by complex world and by simulation of a light ray.
//! 
//! The world is complex because objects have many colors, those colors change with lighting and reflection properties of the materials they are made of. 
//! Also, those properties interact with each other (light sources act in tandem on an object, light reflects from one object to another etc..). 
//! We do light ray simulation is by modeling how a ray of light behaves in the world. Direction at which it meets an object, how it is reflect from the object, how it eventually hits our eye etc...
//!
//! To see how compelx this can get, let's take a look at our goal with this project. We want to generate the following image: 
//! <p style="text-align:center;"><img src="https://raw.githubusercontent.com/ssloy/tinyraytracer/homework_assignment/out-envmap-duck.jpg"  width="500"/></p>
//!
//! ## How we trace rays?
//! Since we are going to project from 3 to 2 dimension we need to pick a viewing angle (projection plane). 
//! This plane is going to We going to call this viewing angle a camre.
//! 
//! [Ray-Sphere intersection](struct.Sphere.html#method.ray_intersect)
//!
//!
//! ## Question for future explorations
//! ### What if we have millions of objects in a scene?
//! I guess you can avoid checking most of the objects and limit your intersection checks based on light rays' direction. How is it done in actual ray tracers?

mod voxel;

extern crate image;

use std::f32::consts::FRAC_2_PI;
use voxel::Vox;
/// A sphere is a 3-D ball, it has a center point and a radius.
struct Sphere {
    center: Vox,
    radius: f32,
    material: Material,
}

#[derive(Clone, Copy)]
struct Material {
    color: (f32, f32, f32),
    pixel: image::Rgb<u8>,
}

impl Material {
    fn _to_pixel(color: (f32, f32, f32)) -> image::Rgb<u8> {
        let (r, g, b) = color;
        image::Rgb([(255. * r) as u8, (255. * g) as u8, (255. * b) as u8])
    }

    fn new(color: (f32, f32, f32)) -> Self {
        let pixel = Self::_to_pixel(color);
        Self { color, pixel }
    }

    fn adjust_light(&mut self, intensity: f32) {
        let (r, g, b) = self.color;
        self.color = (
            (r * intensity).max(0.).min(1.),
            (g * intensity).max(0.).min(1.),
            (b * intensity).max(0.).min(1.),
        );
        self.pixel = Self::_to_pixel(self.color);
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new((0.2, 0.7, 0.8))
    }
}

#[derive(Clone, Copy)]
struct LightSource {
    position: Vox,
    intensity: f32,
}

struct LightRay {
    origin: Vox,
    /// Unit norm direction vector
    direction: Vox
}

impl LightRay {
    fn new(dir: Vox) -> Self{
        Self{origin: Vox::orig(), direction: dir.normalized()}
    }

    fn set_origin(&mut self, origin: Vox) {
        self.origin = origin;
    }

    fn walk_dir(&self, distance: f32) -> Vox {
        self.origin + self.direction.walk_dir(distance)
    }
}

/// Point where our ray hits and object.
enum HitPoint {
    Point(Vox),
    None,
}

impl Sphere {
    /// We need to determin if a ray of light hits a specific object or not. This function conatins the logic of how to determine that.
    /// In case of a sphere it's pretty easy, we need to project the center of the sphere on the ray of light and see if the projection is inside the sphere
    fn ray_intersect(&self, ray: &LightRay) -> HitPoint {
        let v = self.center - ray.origin;
        // let u = direction.normalized();

        let dist_orig_proj = v.dot(&ray.direction);

        if dist_orig_proj < 0. {
            return HitPoint::None;
        }
        let proj = ray.walk_dir(dist_orig_proj);

        let dist_ctr_proj = (self.center - proj).l2();

        if dist_ctr_proj > self.radius {
            return HitPoint::None;
        }

        let dist_proj_intersect1 = (self.radius.powf(2.0) - dist_ctr_proj.powf(2.)).sqrt();

        match (
            dist_orig_proj - dist_proj_intersect1,
            dist_orig_proj + dist_proj_intersect1,
        ) {
            (o_i1, _) if o_i1 > 0. => HitPoint::Point(ray.walk_dir(o_i1)),
            // Origin is inside the sphere
            // Assuming light can move thorugh sphere we'll see the other intersection point
            (_, o_i2) if o_i2 > 0. => HitPoint::Point(ray.walk_dir(o_i2)),
            _ => HitPoint::None,
        }
    }
}

/// This is the light ray simulation. We go over the objects in the scene and check if our light ray intersect with them.
/// If there is an intersection, we get the point of intersection and assign the color of the object the ray intersect with. 
/// Next we use the point of intersection and the lighting source in the scene to determine how lighting should affect the color at intersection point.
fn cast_ray(ray: LightRay, scene: &[Sphere], lights: &[LightSource]) -> Material {
    let mut dist = f32::MAX;
    let mut hit_point: Option<Vox> = None;
    let mut normal = Vox::orig();
    let mut material = Material::default();

    for s in scene.iter() {
        match s.ray_intersect(&ray) {
            // Hit is the point where our ray hits the sphere
            HitPoint::Point(p) if (p - ray.origin).l2() < dist => {
                dist = (p - ray.origin).l2();
                material = s.material;
                normal = (p - s.center).normalized();
                hit_point = Some(p);
            }
            _ => continue,
        }
    }

    if let Some(p) = hit_point {
        let mut light_intensity = 0f32;
        for cur in lights.iter() {
            let ldir = (cur.position - p).normalized();
            // dbg!(ldir.dot(&normal), normal.dot(&ldir), normal.l2(), ldir.l2());
            let psi_ = ldir.dot(&normal).max(0.);
            light_intensity += cur.intensity * psi_;
        }
        material.adjust_light(light_intensity);
    }

    material
}

/// This function builds an image by simulatin light rays.
/// Each pixel of an image is translated into a light ray. For each pixel, the light ray simulation returns the color the pixel should get.
fn render(spehres: Vec<Sphere>, lights: Vec<LightSource>, output: &str) {
    let imgx = 1024;
    let imgy = 768;
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    let width = imgx as f32;
    let height = imgy as f32;
    let wh_ratio = width / height;
    let tan_fov = FRAC_2_PI.tan();

    // Iterate over the coordinates and pixels of the image
    for (i, j, pixel) in imgbuf.enumerate_pixels_mut() {
        let rel_w = (i as f32 + 0.5) / width;
        let rel_h = (j as f32 + 0.5) / height;

        let x = (2.0 * rel_w - 1.0) * tan_fov * wh_ratio;
        let y = -(2.0 * rel_h - 1.0) * tan_fov;

        let dir = Vox::new((x, y, -1.0)).normalized();

        let ray = LightRay::new(dir);

        *pixel = cast_ray(ray, &spehres, &lights).pixel;
    }

    imgbuf.save(output);
}

fn main() {
    let ivory = Material::new((0.4, 0.4, 0.3));
    let red_rubber = Material::new((0.3, 0.1, 0.1));

    let s = Sphere {
        center: Vox::new((-3., 0., -16.)),
        radius: 2.0,
        material: ivory,
    };

    let s2 = Sphere {
        center: Vox::new((-1., -1.5, -12.)),
        radius: 2.0,
        material: red_rubber,
    };

    let s3 = Sphere {
        center: Vox::new((1.5, -0.5, -18.)),
        radius: 3.0,
        material: red_rubber,
    };

    let s4 = Sphere {
        center: Vox::new((7., 5., -18.)),
        radius: 4.0,
        material: ivory,
    };

    let light = LightSource {
        position: Vox::new((-20., 20., 20.)),
        intensity: 1.5,
    };

    render(vec![s, s2, s3, s4], vec![light], "test.png");
}
