use crate::interval;
use crate::interval::Interval;
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;

#[derive(Clone, Copy, Default)]
pub struct Aabb {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}
 
impl Aabb {
    pub const fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }
    
    pub fn from_intervals(x: Interval, y: Interval, z: Interval) -> Self {
        let mut a = Self { x, y, z };
        a.pad_to_minimums();
        a
    }

    pub fn from_points(a: Point3, b: Point3) -> Self {
        // Treat the two points a and b as extrema for the bounding box, so we don't
        // require a particular minimum/maximum coordinate order.
        let x = if a.x() <= b.x() { Interval::new(a.x(), b.x()) } else { Interval::new(b.x(), a.x()) };
        let y = if a.y() <= b.y() { Interval::new(a.y(), b.y()) } else { Interval::new(b.y(), a.y()) };
        let z = if a.z() <= b.z() { Interval::new(a.z(), b.z()) } else { Interval::new(b.z(), a.z()) };
        let mut bbox = Self { x, y, z };
        bbox.pad_to_minimums();
        bbox
    }
    
    pub fn from_aabbs(box0: Aabb, box1: Aabb) -> Self {
        Self {
            x: Interval::from_intervals(box0.x, box1.x),
            y: Interval::from_intervals(box0.y, box1.y),
            z: Interval::from_intervals(box0.z, box1.z),
        }
    }

    // Adjust the AABB so that no side is narrower than some delta, padding if necessary.
    fn pad_to_minimums(&mut self) {
        let delta = 0.0001;
        if self.x.size() < delta { self.x = self.x.expand(delta); }
        if self.y.size() < delta { self.y = self.y.expand(delta); }
        if self.z.size() < delta { self.z = self.z.expand(delta); }
    }

    pub fn axis_interval(&self, n: usize) -> Interval {
        match n {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }

    pub fn hit(&self, r: &Ray, mut ray_t: Interval) -> bool {
        for axis in 0..3 {
            let ax = self.axis_interval(axis);
            let adinv = 1.0 / r.direction().get_index(axis);
            let t0 = (ax.min - r.origin().get_index(axis)) * adinv;
            let t1 = (ax.max - r.origin().get_index(axis)) * adinv;
            if t0 < t1 {
                if t0 > ray_t.min { ray_t.min = t0; }
                if t1 < ray_t.max { ray_t.max = t1; }
            } else {
                if t1 > ray_t.min { ray_t.min = t1; }
                if t0 < ray_t.max { ray_t.max = t0; }
            }
            if ray_t.max <= ray_t.min {
                return false;
            }
        }
        true
    }

    pub fn longest_axis(&self) -> usize {
        // Returns the index of the longest axis of the bounding box.
        if self.x.size() > self.y.size() {
            if self.x.size() > self.z.size() { 0 } else { 2 }
        } else {
            if self.y.size() > self.z.size() { 1 } else { 2 }
        }
    }
}

impl std::ops::Add<Vec3> for Aabb {
    type Output = Aabb;
    fn add(self, offset: Vec3) -> Aabb {
        Aabb::from_intervals(
            self.x + offset.x(),
            self.y + offset.y(),
            self.z + offset.z(),
        )
    }
}
 
impl std::ops::Add<Aabb> for Vec3 {
    type Output = Aabb;
    fn add(self, bbox: Aabb) -> Aabb {
        bbox + self
    }
}

pub const EMPTY : Aabb = Aabb::new(interval::EMPTY, interval::EMPTY, interval::EMPTY);
