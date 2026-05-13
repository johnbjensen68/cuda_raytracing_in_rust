use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::aabb::Aabb;
use std::rc::Rc;

#[derive(Default)]
pub struct HittableList {
    pub objects: Vec<Rc<dyn Hittable>>,
    bbox: Aabb,
}
 
impl HittableList {
    pub fn new() -> HittableList {
        Default::default()
    }
 
    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.bbox = Aabb::from_aabbs(self.bbox, object.bounding_box());
        self.objects.push(Rc::from(object));
    }
}
 
impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut temp_rec = None;
        let mut closest_so_far = t_max;
 
        for object in &self.objects {
            if let Some(rec) = object.hit(ray, t_min, closest_so_far) {
                closest_so_far = rec.t;
                temp_rec = Some(rec);
            }
        }
 
        temp_rec
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}