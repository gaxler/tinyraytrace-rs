//! In our world we have Spheres, Light Sources, Light Rays ang Materials.

use crate::vectors::Vox;
/// A sphere is a 3-D ball, it has a center point and a radius.
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vox,
    pub radius: f32,
    pub material: Material,
}

/// Point where our ray hits and object.
pub enum HitPoint {
    Point(Vox),
    None,
}

impl Sphere {
    /// We need to determine if a ray of light hits a specific object or not. This function contains the logic of how to determine that.
    /// In case of a sphere it's pretty easy, we need to project the center of the sphere on the ray of light and see if the projection is inside the sphere
    pub fn ray_intersect(&self, ray: &LightRay) -> HitPoint {
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

#[derive(Clone, Copy)]
pub struct LightSource {
    pub position: Vox,
    pub intensity: f32,
}

#[derive(Clone, Copy)]
pub struct LightRay {
    pub origin: Vox,
    /// Unit norm direction vector
    pub direction: Vox,
}

impl LightRay {
    pub fn new(dir: Vox) -> Self {
        Self {
            origin: Vox::orig(),
            direction: dir.normalized(),
        }
    }

    pub fn set_origin(mut self, origin: Vox) -> Self {
        self.origin = origin;
        self
    }

    pub fn walk_dir(&self, distance: f32) -> Vox {
        self.origin + self.direction.mult(distance)
    }
}

/// Material represents the color and light reflecting properties. (Open the struct page to see images)
///
///This is something completely new to me. The wikipedia article is interesting [Phong Reflection Model](https://en.wikipedia.org/wiki/Phong_reflection_model).
///Particularly this image <p>![](https://upload.wikimedia.org/wikipedia/commons/thumb/0/01/Blinn_Vectors.svg/330px-Blinn_Vectors.svg.png)</p>
///Another image that provides good explanation about diffused and specular reflection is this: <p> ![](https://upload.wikimedia.org/wikipedia/commons/thumb/b/bd/Lambert2.gif/330px-Lambert2.gif)</p>
#[derive(Clone, Copy, Debug)]
pub struct Material {
    color: (f32, f32, f32),
    pub pixel: image::Rgb<u8>,
    /// How strong this material reflects direct light
    pub specular_exponent: f32,
    /// Whiteness of an object
    albedo: (f32, f32),
    reflection_mixing_coef: f32,
}

impl Material {
    fn _to_pixel(color: (f32, f32, f32)) -> image::Rgb<u8> {
        let (r, g, b) = color;
        image::Rgb([(255. * r) as u8, (255. * g) as u8, (255. * b) as u8])
    }

    pub fn new(
        color: (f32, f32, f32),
        albedo: (f32, f32),
        specular_exponent: f32,
        reflection_mixing_coef: f32,
    ) -> Self {
        let pixel = Self::_to_pixel(color);
        Self {
            color,
            pixel,
            albedo,
            specular_exponent,
            reflection_mixing_coef,
        }
    }

    pub fn adjust_light(mut self, diffuse: f32, specular: f32) -> Self {
        let (r, g, b) = self.color;
        let diff_albedo = diffuse * self.albedo.0;
        let white_shift = specular * self.albedo.1;

        self.color = (
            (r * diff_albedo + white_shift).max(0.).min(1.),
            (g * diff_albedo + white_shift).max(0.).min(1.),
            (b * diff_albedo + white_shift).max(0.).min(1.),
        );

        self.pixel = Self::_to_pixel(self.color);
        self
    }

    /// Mix two materials color together, by the amount of reflectiveness of the first material.
    pub fn mix_reflection(mut self, other: Material) -> Self {
        // let (r1, g1, b1) = dbg!(self.color);
        let (r1, g1, b1) = self.color;
        let (r2, g2, b2) = other.color;

        let mixed_color = (
            (r1 + self.reflection_mixing_coef * r2).max(0.).min(1.),
            (g1 + self.reflection_mixing_coef * g2).max(0.).min(1.),
            (b1 + self.reflection_mixing_coef * b2).max(0.).min(1.),
        );

        self.color = mixed_color;
        self.pixel = Self::_to_pixel(self.color);
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new((0.2, 0.7, 0.8), (1.0, 0.0), 1.0, 0.0)
    }
}
