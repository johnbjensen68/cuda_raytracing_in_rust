use crate::ray::Ray;
use crate::vec3::{self, Point3, Vec3};
use std::rc::Rc;
use crate::material::Material;
use crate::aabb::Aabb;

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Rc<dyn Material>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}
 
impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = vec3::dot(r.direction(), outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}
 
pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self) -> Aabb;
}

pub struct Translate {
    object: Box<dyn Hittable>,
    offset: Vec3,
    bbox: Aabb,
}

impl Translate {
    pub fn new(object: Box<dyn Hittable>, offset: Vec3) -> Self {
        let bbox = object.bounding_box() + offset;
        Self { object, offset, bbox }
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // Move the ray backwards by the offset
        let offset_r = Ray::new(r.origin() - self.offset, r.direction(), r.time());
 
        // Determine whether an intersection exists along the offset ray (and if so, where)
        let Some(mut rec) = self.object.hit(&offset_r, t_min, t_max) else  {
            return None;
        };
 
        // Move the intersection point forwards by the offset
        rec.p += self.offset;
 
        Some(rec)
    }

    fn bounding_box(&self) -> Aabb { self.bbox }
}


pub struct RotateY {
    object: Box<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Aabb,
}
 
impl RotateY {
    pub fn new(object: Box<dyn Hittable>, angle: f64) -> Self {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox = object.bounding_box();
 
        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
 
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f64 * bbox.x.max + (1 - i) as f64 * bbox.x.min;
                    let y = j as f64 * bbox.y.max + (1 - j) as f64 * bbox.y.min;
                    let z = k as f64 * bbox.z.max + (1 - k) as f64 * bbox.z.min;
 
                    let newx =  cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;
 
                    let tester = Vec3::new(newx, y, newz);
 
                    for c in 0..3 {
                        min.set_index(c, min.get_index(c).min(tester.get_index(c)));
                        max.set_index(c, max.get_index(c).max(tester.get_index(c)));
                    }
                }
            }
        }
 
        let bbox = Aabb::from_points(min, max);
        Self { object, sin_theta, cos_theta, bbox }
    }
}
 
 impl Hittable for RotateY {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
 
        // Transform the ray from world space to object space.
 
        let origin = Point3::new(
            (self.cos_theta * r.origin().x()) - (self.sin_theta * r.origin().z()),
            r.origin().y(),
            (self.sin_theta * r.origin().x()) + (self.cos_theta * r.origin().z()),
        );
 
        let direction = Vec3::new(
            (self.cos_theta * r.direction().x()) - (self.sin_theta * r.direction().z()),
            r.direction().y(),
            (self.sin_theta * r.direction().x()) + (self.cos_theta * r.direction().z()),
        );
 
        let rotated_r = Ray::new(origin, direction, r.time());
 
        // Determine whether an intersection exists in object space (and if so, where).
         let Some(mut rec) = self.object.hit(&rotated_r, t_min, t_max) else  {
            return None;
        };
 
        // Transform the intersection from object space back to world space.
 
        rec.p = Point3::new(
            (self.cos_theta * rec.p.x()) + (self.sin_theta * rec.p.z()),
            rec.p.y(),
            (-self.sin_theta * rec.p.x()) + (self.cos_theta * rec.p.z()),
        );
 
        rec.normal = Vec3::new(
            (self.cos_theta * rec.normal.x()) + (self.sin_theta * rec.normal.z()),
            rec.normal.y(),
            (-self.sin_theta * rec.normal.x()) + (self.cos_theta * rec.normal.z()),
        );
 
        Some(rec)
    }
 
    fn bounding_box(&self) -> Aabb { self.bbox }
}