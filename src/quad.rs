use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::vec3::{Point3, Vec3};
use crate::aabb::Aabb;
use crate::material::Material;
use crate::hittable::{Hittable, HitRecord};
use crate::ray::Ray;
use crate::vec3;

use std::rc::Rc;

pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: Rc<dyn Material>,
    bbox: Aabb,
    normal: Vec3,
    d: f64,
}
 
impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Rc<dyn Material>) -> Self {
        let n = vec3::cross(u,v);
        let normal = vec3::unit_vector(n);
        let d = vec3::dot(normal, q);
        let w = n / vec3::dot(n, n);

        let mut quad = Self {
            q, u, v, w, mat,
            bbox: Aabb::default(),
            normal, d,
        };
        quad.set_bounding_box();
        quad
    }
 
    fn set_bounding_box(&mut self) {
        // Compute the bounding box of all four vertices.
        let bbox_diagonal1 = Aabb::from_points(self.q, self.q + self.u + self.v);
        let bbox_diagonal2 = Aabb::from_points(self.q + self.u, self.q + self.v);
        self.bbox = Aabb::from_aabbs(bbox_diagonal1, bbox_diagonal2);
    }

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)> {
        let unit_interval = Interval::new(0.0, 1.0);
        // Given the hit point in plane coordinates, return false if it is outside the
        // primitive, otherwise set the hit record UV coordinates and return true.
 
        if !unit_interval.contains(a) || !unit_interval.contains(b) {
            return None;
        }
        return Some((a, b));
    }
}
 
impl Hittable for Quad {
    fn bounding_box(&self) -> Aabb { self.bbox }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let ray_t = Interval::new(t_min, t_max);
        let denom = vec3::dot(self.normal, r.direction());
 
        // No hit if the ray is parallel to the plane.
        if denom.abs() < 1e-8 {
            return None;
        }
 
        // Return false if the hit point parameter t is outside the ray interval.
        let t = (self.d - vec3::dot(self.normal, r.origin())) / denom;
        if !ray_t.contains(t) {
            return None;
        }
 
        // Determine if the hit point lies within the planar shape using its plane coordinates.
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let alpha = vec3::dot(self.w, vec3::cross(planar_hitpt_vector, self.v));
        let beta  = vec3::dot(self.w, vec3::cross(self.u, planar_hitpt_vector));
 
        let interior = Self::is_interior(alpha, beta)?;

        let mut rec = HitRecord {
            t,
            p: intersection,
            mat: self.mat.clone(),
            normal: self.normal,
            front_face: Default::default(),
            u: interior.0,
            v: interior.1,
        };
        rec.set_face_normal(r, self.normal);
        Some(rec)
    }
}

pub fn make_box(a: Point3, b: Point3, mat: Rc<dyn Material>) -> Box<HittableList> {
    // Returns the 3D box (six sides) that contains the two opposite vertices a & b.
 
    let mut sides = HittableList::new();
 
    // Construct the two opposite vertices with the minimum and maximum coordinates.
    let min = Point3::new(a.x().min(b.x()), a.y().min(b.y()), a.z().min(b.z()));
    let max = Point3::new(a.x().max(b.x()), a.y().max(b.y()), a.z().max(b.z()));
 
    let dx = Vec3::new(max.x() - min.x(), 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y() - min.y(), 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z() - min.z());
 
    sides.add(Box::new(Quad::new(Point3::new(min.x(), min.y(), max.z()),  dx,  dy, mat.clone()))); // front
    sides.add(Box::new(Quad::new(Point3::new(max.x(), min.y(), max.z()), -dz,  dy, mat.clone()))); // right
    sides.add(Box::new(Quad::new(Point3::new(max.x(), min.y(), min.z()), -dx,  dy, mat.clone()))); // back
    sides.add(Box::new(Quad::new(Point3::new(min.x(), min.y(), min.z()),  dz,  dy, mat.clone()))); // left
    sides.add(Box::new(Quad::new(Point3::new(min.x(), max.y(), max.z()),  dx, -dz, mat.clone()))); // top
    sides.add(Box::new(Quad::new(Point3::new(min.x(), min.y(), min.z()),  dx,  dz, mat))); // bottom
 
    Box::new(sides)
}