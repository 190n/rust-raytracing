use std::rc::Rc;

use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;

pub struct HittableList {
	objects: Vec<Rc<dyn Hittable>>,
}

impl HittableList {
	pub fn new() -> Self {
		Self {
			objects: Vec::new(),
		}
	}

	pub fn add(&mut self, object: Rc<dyn Hittable>) {
		self.objects.push(object);
	}
}

impl Hittable for HittableList {
	fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
		let mut temp_rec: Option<HitRecord> = None;
		let mut closest_so_far = t_max;

		for o in &self.objects {
			if let Some(rec) = o.hit(r, t_min, closest_so_far) {
				closest_so_far = rec.t;
				temp_rec = Some(rec);
			}
		}

		temp_rec
	}
}
