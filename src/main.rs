//! This is a basic ray tracer. It is based on [Understandable RayTracing in 256 lines of bare C++](https://github.com/ssloy/tinyraytracer/wiki)
//! Coming from a computer vision background, I had some idea how ray tracing might work, this is for me to actually understand it.
//! To make it more fun, I wanna try and write documentation that's going to cover some fundamental concepts as part of it.
//!
//! ( _This is a work in progress and I don't add documentation in a linear fashion, so it might be a bit early to try and read it_)
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

mod vectors;
mod blocks;

extern crate image;

use std::f32::consts::FRAC_2_PI;
use vectors::Vox;
use blocks::*;


struct IntersectionState {
    hit_point: Vox,
    normal: Vox,
    material: Material,
    ray: LightRay,
}

/// This is the light ray simulation. We go over the objects in the scene and check if our light ray intersect with them.
/// If there is an intersection, we get the point of intersection and assign the color of the object the ray intersect with.
/// Next we use the point of intersection and the lighting source in the scene to determine how lighting should affect the color at intersection point.
fn cast_ray(ray: LightRay, scene: &[Sphere]) -> Option<IntersectionState> {
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

    // The question mark checks if hit_point is None or Some if it is None then function returns None otherwise it unpacks the Some
    Some(IntersectionState {
        hit_point: hit_point?,
        normal,
        material,
        ray,
    })
}

/// Shadow is like a negative light, we "cast a ray of shadow" for a certain hit point and light source.
/// If the shadow ray hits the object, we know that the object is in shadow and we can't see the light source. ([Github Copilot](https://copilot.github.com/) wrote this line for me, how cool is that?)
fn light_is_shadowed(hit_point: Vox, hit_normal: Vox, light_position: Vox, scene: &[Sphere]) -> bool {
    
    let ldir = (light_position - hit_point).normalized();
    let ldist = (light_position - hit_point).l2();

    let shadow_shift = 0.001f32.copysign(ldir.dot(&hit_normal));
    let shadow_orig = hit_point + hit_normal.mult(shadow_shift);

    
    let shadow_ray = LightRay::new(ldir).set_origin(shadow_orig);

    if let Some(shadow) = cast_ray(shadow_ray, scene) {
        if (shadow.hit_point - shadow_orig).l2() < ldist {return true;} 
    }
    false

}

fn light_intersect(
    intersection: IntersectionState,
    scene: &[Sphere],
    lights: &[LightSource],
) -> Material {
    let (normal, p, ray) = (
        intersection.normal,
        intersection.hit_point,
        intersection.ray,
    );

    let mut material = intersection.material;

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
            .powf(material.specular_exponent);

        diffuse += cur.intensity * diff_coef;
        specular += cur.intensity * spec_coef;
            
        
    }

    material.adjust_light(diffuse, specular);
    material
}

/// This function builds an image by simulating light rays.
/// Each pixel of an image is translated into a light ray. For each pixel, the light ray simulation returns the color the pixel should get.
fn render(spheres: Vec<Sphere>, lights: Vec<LightSource>, output: &str) {
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

        let pix_value = match cast_ray(ray, &spheres) {
            None => Material::default().pixel,
            Some(intersection) => light_intersect(intersection, &spheres, &lights).pixel,
        };

        *pixel = pix_value;

        // *pixel = cast_ray(ray, &spheres, &lights).pixel;
    }

    imgbuf.save(output).expect("Failed saving canvas");
}

fn main() {
    let ivory = Material::new((0.4, 0.4, 0.3), (0.6, 0.3), 50.);
    // let ivory = Material::new((0.4, 0.4, 0.3), (1., 0.3), 50.);
    let red_rubber = Material::new((0.3, 0.1, 0.1), (0.9, 0.1), 10.);
    // let red_rubber = Material::new((0.3, 0.1, 0.1),(1., 0.1), 10.);

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

    render(
        vec![s, s2, s3, s4],
        vec![light],
        "static/assets/current.png",
    );
}
