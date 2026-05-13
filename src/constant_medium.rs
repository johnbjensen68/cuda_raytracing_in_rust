use std::rc::Rc;

use crate::{aabb::Aabb, color::Color, common, hittable::{HitRecord, Hittable}, material::{Isotropic, Material}, ray::Ray, texture::Texture, vec3::Vec3};

pub struct ConstantMedium {
    boundary: Box<dyn Hittable>,
    neg_inv_density: f64,
    phase_function: Rc<dyn Material>,
}
 
impl ConstantMedium {
    pub fn new(boundary: Box<dyn Hittable>, density: f64, tex: Rc<dyn Texture>) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Rc::new(Isotropic::new(tex)),
        }
    }
 
    pub fn from_color(boundary: Box<dyn Hittable>, density: f64, albedo: Color) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Rc::new(Isotropic::from_color(albedo)),
        }
    }
}
 
impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let Some(mut rec1) = self.boundary.hit(r, f64::NEG_INFINITY, f64::INFINITY) else {
            return None;
        };
        let Some(mut rec2) = self.boundary.hit(r, rec1.t + 0.0001, f64::INFINITY) else {
            return None;
        };
 
        if rec1.t < t_min { rec1.t = t_min; }
        if rec2.t > t_max { rec2.t = t_max; }
 
        if rec1.t >= rec2.t {
            return None;
        }
 
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }
 
        let ray_length = r.direction().length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * common::random_double().ln();
 
        if hit_distance > distance_inside_boundary {
            return None;
        }
 
        let t = rec1.t + hit_distance / ray_length;
        Some(HitRecord {
            t,
            p: r.at(t),
            normal: Vec3::new(1.0, 0.0, 0.0), // arbitrary
            front_face: true,                 // also arbitrary
            mat: self.phase_function.clone(),
            u: 0.0,
            v: 0.0,
        })
    }
 
    fn bounding_box(&self) -> Aabb {
        self.boundary.bounding_box()
    }
}