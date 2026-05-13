use crate::hittable::{HitRecord, Hittable};
use crate::aabb::Aabb;
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::ray::Ray;
use std::rc::Rc;
use std::cmp::Ordering;

pub struct BvhNode {
    left: Rc<dyn Hittable>,
    right: Rc<dyn Hittable>,
    bbox: Aabb,
}
 
 fn box_compare(a: &dyn Hittable, b: &dyn Hittable, axis: usize) -> Ordering {
    let a_axis_interval = a.bounding_box().axis_interval(axis);
    let b_axis_interval = b.bounding_box().axis_interval(axis);
    a_axis_interval.min.partial_cmp(&b_axis_interval.min).unwrap_or(Ordering::Equal)
}

impl BvhNode {
    pub fn new(mut hl: HittableList) -> Self {
        let l = hl.objects.len();
        BvhNode::internal_new(&mut hl.objects, 0, l)
    }
    pub fn internal_new(objects: &mut Vec<Rc<dyn Hittable>>, start: usize, end: usize) -> Self {
        // Build the bounding box of the span of source objects.
        let mut bbox = crate::aabb::EMPTY;
        for i in start..end {
            bbox = Aabb::from_aabbs(bbox, objects[i].bounding_box());
        }
 
        let axis = bbox.longest_axis();
        let object_span = end - start;
        let (left, right): (Rc<dyn Hittable>, Rc<dyn Hittable>) = if object_span == 1 {
            (objects[start].clone(), objects[start].clone())
        } else if object_span == 2 {
            let (a, b) = (objects[start].clone(), objects[start + 1].clone());
            if box_compare(&*a, &*b, axis) == Ordering::Less {
                (a, b)
            } else {
                (b, a)
            }
        } else {
            objects[start..end].sort_by(|a, b| box_compare(&**a, &**b, axis));
            let mid = start + object_span / 2;
            (
                Rc::new(BvhNode::internal_new(objects, start, mid)),
                Rc::new(BvhNode::internal_new(objects, mid, end)),
            )
        };
        let bbox = Aabb::from_aabbs(left.bounding_box(), right.bounding_box());
        Self { left, right, bbox }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let ray_t = Interval::new(t_min, t_max);
        if !self.bbox.hit(r, ray_t) {
            return None;
        }
        let hit_left = self.left.hit(r, t_min, t_max);
        let hit_right = self.right.hit(
            r,
            t_min,
            hit_left.as_ref().map(|x| x.t ).unwrap_or(t_max),
        );
        hit_right.or(hit_left)
    }
 
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
