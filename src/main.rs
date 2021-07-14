//! This is a basic ray tracer. It is based on [Understandable RayTracing in 256 lines of bare C++](https://github.com/ssloy/tinyraytracer/wiki)
//! Coming from a computer vision background, I had some idea how ray tracing might work, this is for me to actually understand it.
//! To make it more fun, I wanna try and write documentation that's going to cover some fundamental concepts as part of it.
//!
//! ( _This is a work in progress and I don't add documentation in a linear fashion, so it might be a bit early to try and read it_)
//! If there is anything you want to fix/comment on/request etc. [Github Discussions](https://github.com/gaxler/tinyraytrace-rs/discussions/) is the best way to get in touch
//!
//! ## What is Ray Tracing?
//! (_caveat: I'm learning about it pretty much for the first time as I implement this._)
//!
//!
//! We have a 3-D model of some object in the world, and we want to express this model as a 2-D computer image. To construct our image we need to decide what color to assign to each pixel of the image.
//! Since the real world is complex we don't have an easy method to determine the color we should assign to each image pixel and we relay on simulations of light rays.
//! Now, lets say more about what we mean by complex world and by simulation of a light ray.
//!
//! The world is complex because objects have many colors, those colors change with lighting and reflection properties of the materials they are made of.
//! Also, those properties interact with each other (light sources act in tandem on an object, light reflects from one object to another etc..).
//! We do light ray simulation is by modeling how a ray of light behaves in the world. Direction at which it meets an object, how it is reflect from the object, how it eventually hits our eye etc...
//!
//! To see how complex this can get, let's take a look at our goal with this project vs. where we are now:
//! <p style="text-align:center;"><img src="https://raw.githubusercontent.com/ssloy/tinyraytracer/homework_assignment/out-envmap-duck.jpg"  width="400"/>
//! <img src="https://raw.githubusercontent.com/gaxler/tinyraytrace-rs/main/static/assets/current.png"  width="400"/>
//!</p>
//!
//! ## How we trace rays?
//! Since we are going to project from 3 to 2 dimension we need to pick a viewing angle (projection plane).
//! This plane is going to We going to call this viewing angle a camera.
//!
//! [Ray-Sphere intersection](struct.Sphere.html#method.ray_intersect)
//!
//!
//! ## Question for future explorations
//! ### What if we have millions of objects in a scene?
//! I guess you can avoid checking most of the objects and limit your intersection checks based on light rays' direction. How is it done in actual ray tracers?
//! ### How to make ray tracing differentiable?
//! Quick search got me this [paper](https://people.csail.mit.edu/tzumao/diffrt/)

mod blocks;
mod vectors;

extern crate image;

use blocks::*;
use std::f32::consts::FRAC_2_PI;
use vectors::Vec3;

const DEFAULT_JITTER: f32 = 0.001;
const MAX_RAY_BOUNCES: u32 = 4;
const CANVAS_WIDTH_HEIGHT: (u32, u32) = (1024, 768);

type SceneObject = Box<dyn RayCollision>;

struct CollisionState {
    hit_point: Vec3,
    normal: Vec3,
    material: Material,
    ray: Ray,
}

impl CollisionState {
    fn _jitter(&self, dir: Vec3, jitter: f32) -> Vec3 {
        jitter_along_normal(self.hit_point, dir, self.normal, jitter)
    }

    fn reflected_ray(&self, jitter: f32) -> Ray {
        let reflect_dir = self.ray.direction.reflect(self.normal);
        let ref_orig = self._jitter(reflect_dir, jitter);

        Ray::new(reflect_dir).set_origin(ref_orig)
    }

    fn refracted_ray(&self, jitter: f32) -> Ray {
        let refract_dir = self
            .ray
            .direction
            .refract(self.normal, self.material.refraction_index)
            .normalized();
        let ref_orig = self._jitter(refract_dir, jitter);

        Ray::new(refract_dir).set_origin(ref_orig)
    }
}

/// This is the light ray simulation. We go over the objects in the scene and check if our light ray intersect with them.
/// If there is an intersection, we get the point of intersection and assign the color of the object the ray intersect with.
/// Next we use the point of intersection and the lighting source in the scene to determine how lighting should affect the color at intersection point.
fn cast_ray(ray: Ray, scene: &[SceneObject]) -> Option<CollisionState> {
    let mut dist = f32::MAX;
    let mut hit_point: Option<Vec3> = None;
    let mut normal = Vec3::orig();
    let mut material = Material::default();

    for s in scene.iter() {
        match s.ray_intersect(&ray) {
            // Hit is the point where our ray hits the sphere
            HitPoint::Point(p) if (p - ray.origin).l2() < dist => {
                dist = (p - ray.origin).l2();
                material = s.collision_material(p);
                normal = s.collision_normal(p);
                hit_point = Some(p);
            }
            _ => continue,
        }
    }

    // The question mark checks if hit_point is None or Some if it is None then function returns None otherwise it unpacks the Some
    Some(CollisionState {
        hit_point: hit_point?,
        normal,
        material,
        ray,
    })
}

/// This function jitters a point along a noraml vector. Why do we need that? [@ssloy explains](https://github.com/ssloy/tinyraytracer/wiki/Part-1:-understandable-raytracing#step-6-shadows):
///"Why is that? It's just that our point lies on the surface of the object, and (except for the question of numerical errors) any ray from this point will intersect the object itself."
fn jitter_along_normal(pt: Vec3, direction: Vec3, normal: Vec3, jitter: f32) -> Vec3 {
    let _shift = jitter.copysign(direction.dot(&normal));
    pt + normal.mult(_shift)
}

/// Shadow is like a negative light, we "cast a ray of shadow" for a certain hit point and light source.
/// If the shadow ray hits the object, we know that the object is in shadow and we can't see the light source. ([Github Copilot](https://copilot.github.com/) wrote this line for me, how cool is that?)
fn light_is_shadowed(
    hit_point: Vec3,
    hit_normal: Vec3,
    light_position: Vec3,
    scene: &[SceneObject],
) -> bool {
    let ldir = (light_position - hit_point).normalized();
    let ldist = (light_position - hit_point).l2();

    let shadow_orig = jitter_along_normal(hit_point, ldir, hit_normal, DEFAULT_JITTER);
    let shadow_ray = Ray::new(ldir).set_origin(shadow_orig);

    if let Some(shadow) = cast_ray(shadow_ray, scene) {
        if (shadow.hit_point - shadow_orig).l2() < ldist {
            return true;
        }
    }
    false
}

fn get_light_adjustments(
    collision: &CollisionState,
    scene: &[SceneObject],
    lights: &[LightSource],
) -> (f32, f32) {
    let (normal, p, ray) = (collision.normal, collision.hit_point, collision.ray);

    let mut diffuse = 0f32;
    let mut specular = 0f32;

    for cur in lights.iter() {
        let ldir = (cur.position - p).normalized();
        let diff_coef = ldir.dot(&normal).max(0.);

        if light_is_shadowed(p, normal, cur.position, scene) {
            continue;
        }

        let spec_coef = ldir
            .reflect(normal)
            .dot(&ray.direction)
            .max(0.)
            .powf(collision.material.specular_exponent);

        diffuse += cur.intensity * diff_coef;
        specular += cur.intensity * spec_coef;
    }

    (diffuse, specular)
    // material.adjust_light(diffuse, specular)
}

/// Our ray of lights don't stay in the same spot. If the hit some reflective material, they bounce off it like a ball.
/// The is a recursive process. We start with a ray of light and cast it through the scene. Every time a ray hits some object and bounces off, well that's a new ray.
/// In real life ( I guess ) this process can go on until light losses energy, here we put a hard limit on the number of bounces.
fn reflective_ray_cast(
    ray: Ray,
    scene: &[SceneObject],
    lights: &[LightSource],
    depth: u32,
) -> Material {
    match cast_ray(ray, scene) {
        Some(collision) if depth < MAX_RAY_BOUNCES => {
            // refLECted ray cast
            let reflected_ = reflective_ray_cast(
                collision.reflected_ray(DEFAULT_JITTER),
                scene,
                lights,
                depth + 1,
            );

            // refRACted ray cast
            let refracted_ = reflective_ray_cast(
                collision.refracted_ray(DEFAULT_JITTER),
                scene,
                lights,
                depth + 1,
            );

            let (diff, spec) = get_light_adjustments(&collision, scene, lights);

            collision
                .material
                .adjust_light(diff, spec)
                .mix_reflection(reflected_)
                .mix_refraction(refracted_)
        }
        Some(intersection) => {
            let (diff, spec) = get_light_adjustments(&intersection, scene, lights);
            intersection.material.adjust_light(diff, spec)
        }
        _ => Material::default(),
    }
}

/// This function builds an image by simulating light rays.
/// Each pixel of an image is translated into a light ray. For each pixel, the light ray simulation returns the color the pixel should get.
fn render(spheres: Vec<SceneObject>, lights: Vec<LightSource>, output: &str) {
    let (imgx, imgy) = CANVAS_WIDTH_HEIGHT;
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

        let dir = Vec3::new((x, y, -1.0)).normalized();

        let ray = Ray::new(dir);

        let reflected_material = reflective_ray_cast(ray, &spheres, &lights, 0);
        *pixel = reflected_material.pixel;
    }

    imgbuf.save(output).expect("Failed saving canvas");
}

struct SphereBuilder {
    spheres: Vec<Sphere>,
}

impl SphereBuilder {
    fn new() -> Self {
        Self { spheres: vec![] }
    }

    fn add(mut self, center: (f32, f32, f32), radius: f32, material: Material) -> Self {
        self.spheres.push(Sphere {
            center: Vec3::new(center),
            radius,
            material,
        });
        self
    }

    fn build(self) -> Vec<Sphere> {
        self.spheres
    }
}

struct LightBuilder {
    lights: Vec<LightSource>,
}

impl LightBuilder {
    fn new() -> Self {
        Self { lights: vec![] }
    }

    fn add(mut self, center: (f32, f32, f32), intensity: f32) -> Self {
        self.lights.push(LightSource {
            position: Vec3::new(center),
            intensity,
        });
        self
    }

    fn build(self) -> Vec<LightSource> {
        self.lights
    }
}

fn main() {
    let w_ivory = (0.6, 0.3, 0.1, 0.0);
    let w_glass = (0., 0.5, 0.1, 0.8);
    let w_rubber = (0.9, 0.1, 0.0, 0.0);
    let w_mirror = (0., 10., 0.8, 0.0);

    let ivory = Material::new((0.4, 0.4, 0.3), w_ivory, 50., 1.0);
    let glass = Material::new((0.6, 0.7, 0.8), w_glass, 125., 1.5);
    let red_rubber = Material::new((0.3, 0.1, 0.1), w_rubber, 10., 1.0);
    let mirror = Material::new((1., 1., 1.), w_mirror, 1425., 1.0);

    let spheres = SphereBuilder::new()
        // .add((-3., -0., -16.), 2.0, ivory)
        // .add((-1., -1.5, -12.), 2.0, glass)
        // .add((1.5, -0.5, -18.), 3.0, red_rubber)
        .add((7., 5., -18.), 4., mirror)
        // .add((-7., -4., -18.), 4., red_rubber)
        .build();

    let mut scene = spheres
        .iter()
        .map(|&v| Box::new(v) as SceneObject)
        .collect::<Vec<SceneObject>>();

    let plain = Rectangle2D::new(
        Vec3::new((0., -4., -1.)),
        Vec3::new((2.,-4., -10.)),
        Vec3::new((1., 0., 0.)),
        red_rubber,
    );

    scene.push(Box::new(plain));

    let lights = LightBuilder::new()
        .add((-20., 20., 20.), 1.5)
        .add((30., 50., -25.), 1.3)
        .add((30., 20., 30.), 1.3)
        .build();

    render(scene, lights, "static/assets/current.png");
}
