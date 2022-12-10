use std::sync::Arc;

use crate::lib::Ray;
use crate::object::{HitRecord, Hittable};
use crate::scene::Aabb;

#[derive(Debug)]
pub struct HittableList {
	objects: Vec<Arc<dyn Hittable>>,
}

impl HittableList {
	pub fn new() -> Self {
		Self {
			objects: Vec::new(),
		}
	}

	pub fn add(&mut self, object: Arc<dyn Hittable>) {
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

	fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
		if self.objects.is_empty() {
			return None;
		}

		let mut temp_box: Option<Aabb> = None;
		for o in &self.objects {
			if let Some(bb) = o.bounding_box(time0, time1) {
				temp_box = if let Some(bb2) = temp_box {
					Some(Aabb::surrounding_box(bb, bb2))
				} else {
					Some(bb)
				};
			} else {
				return None;
			}
		}

		temp_box
	}
}

impl AsRef<[Arc<dyn Hittable>]> for HittableList {
	fn as_ref(&self) -> &[Arc<dyn Hittable>] {
		&self.objects
	}
}
