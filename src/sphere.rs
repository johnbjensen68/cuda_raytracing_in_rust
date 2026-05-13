use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{self, Point3, Vec3};
use crate::aabb::Aabb;
use std::rc::Rc;

pub struct Sphere {
    center: Ray,
    radius: f64,
    mat: Rc<dyn Material>,
    bbox: Aabb,
}
 
impl Sphere {
    pub fn new(cen: Point3, r: f64, m: Rc<dyn Material>) -> Sphere {
        let rvec = Vec3::new(r, r, r);
        let bbox = Aabb::from_points(cen - rvec, cen + rvec);
        Sphere {
            center: Ray::new(cen, Vec3::new(0.0,0.0,0.0), 0.0),
            radius: r,
            mat: m,
            bbox: bbox
        }
    }

    pub fn new_moving(cen1: Point3, cen2: Point3, r: f64, m: Rc<dyn Material>) -> Sphere {
        let rvec = Vec3::new(r, r, r);
        let box1 = Aabb::from_points(cen1 - rvec, cen1 + rvec);
        let box2 = Aabb::from_points(cen2 - rvec, cen2 + rvec);
        let bbox = Aabb::from_aabbs(box1, box2);
        Sphere {
            center: Ray::new(cen1, cen2 - cen1, 0.0),
            radius: r,
            mat: m,
            bbox: bbox
        }
    }


}
 
fn get_sphere_uv(p: &Point3) -> (f64, f64) {
    // p: a given point on the sphere of radius one, centered at the origin.
    // u: angle around the Y axis from X=-1, mapped to [0,1].
    // v: angle from Y=-1 to Y=+1, mapped to [0,1].
    let theta = (-p.y()).acos();
    let phi = (-p.z()).atan2(p.x()) + std::f64::consts::PI;
    let u = phi / (2.0 * std::f64::consts::PI);
    let v = theta / std::f64::consts::PI;
    (u, v)
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let current_center = self.center.at(r.time());
        let oc =  r.origin() - current_center;
        let a = r.direction().length_squared();
        let half_b = vec3::dot(oc, r.direction());
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
 
        let sqrt_d = f64::sqrt(discriminant);
 
        // Find the nearest root that lies in the acceptable range
        let mut root = (-half_b - sqrt_d) / a;
        if root <= t_min || t_max <= root {
            root = (-half_b + sqrt_d) / a;
            if root <= t_min || t_max <= root {
                return None;
            }
        }
 
        let mut rec = HitRecord {
            t: root,
            p: r.at(root),
            mat: self.mat.clone(),
            normal: Default::default(),
            front_face: Default::default(),
            u: 0.0, // For now, since we don't need u or v
            v: 0.0,
        };
        let outward_normal = (rec.p - current_center) / self.radius;
        rec.set_face_normal(r, outward_normal);
        let (u, v) = get_sphere_uv(&outward_normal);
        rec.u = u;
        rec.v = v;
        Some(rec)
    }

     fn bounding_box(&self) -> Aabb {
        self.bbox
     }
}