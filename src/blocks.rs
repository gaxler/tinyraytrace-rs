//! In our world we have Spheres, Light Sources, Light Rays ang Materials.

use crate::vectors::Vec3;

/// We need to determine if a ray of light hits a specific object or not. This trait contains the logic of how to determine that.
pub trait RayCollision {
    fn ray_intersect(&self, ray: &Ray) -> HitPoint;

    fn collision_normal(&self, hit_point: Vec3) -> Vec3;

    fn collision_material(&self, hit_point: Vec3) -> Material;
}

pub struct Plane {
    pub normal: Vec3,
    pub point: Vec3,
}

impl RayCollision for Plane {
    fn ray_intersect(&self, ray: &Ray) -> HitPoint {
        let orig_to_point = self.point - ray.origin;
        let origin_to_plane_dist = self.normal.dot(&orig_to_point);
        let cos_dir_norm = self.normal.dot(&ray.direction);

        if cos_dir_norm * origin_to_plane_dist < 0. {
            HitPoint::None
        } else {
            let dist_to_collision = origin_to_plane_dist / cos_dir_norm;
            HitPoint::Point(ray.walk_dir(dist_to_collision))
        }
    }

    fn collision_normal(&self, hit_point: Vec3) -> Vec3 {
        self.normal
    }

    fn collision_material(&self, hit_point: Vec3) -> Material {
        Material::default()
    }
}

/// 2D rectangle in a 3D space
pub struct Rectangle2D {
    width: Vec3,
    height: Vec3,
    plane: Plane,
    material: Material,
}

impl Rectangle2D {
    /// We need 3 points to define a plane.
    /// Here we use two points on a plane and a vector that is used as the side of the rectangle.
    pub fn new(origin: Vec3, center: Vec3, side_dir: Vec3, material: Material) -> Self {
        // Distance from
        let z = center - origin;
        let e1 = side_dir.normalized();

        let u = z.project_on(&e1);

        let e2 = (z - u).normalized();

        let w = 2. * z.project_on(&e1).l2();
        let h = 2. * z.project_on(&e2).l2();
        let normal = e1.cross(&e2);

        let plane = Plane {
            normal,
            point: origin,
        };

        Self {
            width: e1.mult(w),
            height: e2.mult(h),
            plane,
            material,
        }
    }
}

impl RayCollision for Rectangle2D {
    /// This is easy. We look for plane-ray intersection and check if it is withing the rectangle
    fn ray_intersect(&self, ray: &Ray) -> HitPoint {
        match self.plane.ray_intersect(ray) {
            HitPoint::None => HitPoint::None,
            HitPoint::Point(p) => {
                // plane.point is the origin of the rectangle.
                // rectangle stretches across self.width, self.height
                let d = p - self.plane.point;
                if let (true, true) = (
                    d.project_on(&self.width).l2() <= self.width.l2(),
                    d.project_on(&self.height).l2() <= self.height.l2(),
                ) {
                    HitPoint::Point(p)
                } else {
                    HitPoint::None
                }
            }
        }
    }

    fn collision_normal(&self, hit_point: Vec3) -> Vec3 {
        self.plane.normal
    }

    fn collision_material(&self, hit_point: Vec3) -> Material {
        self.material
    }
}

/// A sphere is a 3-D ball, it has a center point and a radius.
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

/// Point where our ray hits and object.
pub enum HitPoint {
    Point(Vec3),
    None,
}

/// In case of a sphere it's pretty easy, we need to project the center of the sphere on the ray of light and see if the projection is inside the sphere    
impl RayCollision for Sphere {
    fn ray_intersect(&self, ray: &Ray) -> HitPoint {
        let canonical_center = self.center - ray.origin;
        let center_projected_ray = canonical_center.project_on(&ray.direction);

        let dist_ctr_proj = (canonical_center - center_projected_ray).l2();

        if dist_ctr_proj > self.radius {
            return HitPoint::None;
        }

        let dist_proj_intersect1 = (self.radius.powf(2.0) - dist_ctr_proj.powf(2.)).sqrt();

        let dist_orig_proj = canonical_center.dot(&ray.direction);

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

    fn collision_normal(&self, hit_point: Vec3) -> Vec3 {
        (hit_point - self.center).normalized()
    }

    fn collision_material(&self, hit_point: Vec3) -> Material {
        self.material
    }
}

#[derive(Clone, Copy)]
pub struct LightSource {
    pub position: Vec3,
    pub intensity: f32,
}

/// What is the difference between a Vec3 and a Ray? After all Vec3 is a Ray that starts at the origin.
/// Well Ray is infinite length. That's why the direction can be unit norm. Vec3 length is finite (it's norm).
#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    /// Unit norm direction vector
    pub direction: Vec3,
}

impl Ray {
    pub fn new(dir: Vec3) -> Self {
        Self {
            origin: Vec3::orig(),
            direction: dir.normalized(),
        }
    }

    pub fn set_origin(mut self, origin: Vec3) -> Self {
        self.origin = origin;
        self
    }

    pub fn walk_dir(&self, distance: f32) -> Vec3 {
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
    /// How refracting is the material
    pub refraction_index: f32,
    /// Whiteness of an object
    // albedo: (f32, f32),
    diff_mixing_coef: f32,
    spec_mixing_coef: f32,
    reflection_mixing_coef: f32,
    refraction_mixing_coef: f32,
}

type MaterialMixingWeights = (f32, f32, f32, f32);

impl Material {
    fn _to_pixel(color: (f32, f32, f32)) -> image::Rgb<u8> {
        let (r, g, b) = color;
        image::Rgb([(255. * r) as u8, (255. * g) as u8, (255. * b) as u8])
    }

    pub fn new(
        color: (f32, f32, f32),
        weights: MaterialMixingWeights,
        // albedo: (f32, f32),
        specular_exponent: f32,
        refraction_index: f32,
        // reflection_mixing_coef: f32,
        // refraction_mixing_coef: f32,
    ) -> Self {
        let pixel = Self::_to_pixel(color);
        let (diff_mixing_coef, spec_mixing_coef, reflection_mixing_coef, refraction_mixing_coef) =
            weights;
        Self {
            color,
            pixel,
            specular_exponent,
            refraction_index,
            diff_mixing_coef,
            spec_mixing_coef,
            reflection_mixing_coef,
            refraction_mixing_coef,
        }
    }

    pub fn adjust_light(mut self, diffuse: f32, specular: f32) -> Self {
        let (r, g, b) = self.color;
        let diff_albedo = diffuse * self.diff_mixing_coef;
        let white_shift = specular * self.spec_mixing_coef;

        self.color = (
            (r * diff_albedo + white_shift).max(0.).min(1.),
            (g * diff_albedo + white_shift).max(0.).min(1.),
            (b * diff_albedo + white_shift).max(0.).min(1.),
        );

        self.pixel = Self::_to_pixel(self.color);
        self
    }

    fn _mix_materials(mut self, other: Material, coef: f32) -> Self {
        let (r1, g1, b1) = self.color;
        let (r2, g2, b2) = other.color;

        let mixed_color = (
            (r1 + coef * r2).max(0.).min(1.),
            (g1 + coef * g2).max(0.).min(1.),
            (b1 + coef * b2).max(0.).min(1.),
        );

        self.color = mixed_color;
        self.pixel = Self::_to_pixel(self.color);
        self
    }

    /// Mix two materials color together, by the amount of reflectiveness of the first material.
    pub fn mix_reflection(self, other: Material) -> Self {
        self._mix_materials(other, self.reflection_mixing_coef)
    }

    /// Mix two materials color together, by the amount of refraction of the first material.
    pub fn mix_refraction(self, other: Material) -> Self {
        self._mix_materials(other, self.refraction_mixing_coef)
    }
}

impl Default for Material {
    fn default() -> Self {
        let weights: MaterialMixingWeights = (1.0, 0.0, 0.0, 0.);
        Self::new((0.2, 0.7, 0.8), weights, 1.0, 1.0)
    }
}
