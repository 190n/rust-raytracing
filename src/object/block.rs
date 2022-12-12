use std::sync::Arc;

use super::{HitRecord, Hittable, Material, XYRect, XZRect, YZRect};
use crate::lib::{Point3, Ray};
use crate::scene::{Aabb, HittableList};

#[derive(Debug)]
pub struct Block {
	aabb: Aabb,
	sides: HittableList,
}

impl Block {
	pub fn new(p0: Point3, p1: Point3, mat: Arc<dyn Material>) -> Block {
		let mut sides = HittableList::new();

		sides.add(Arc::new(XYRect::new(
			p0.x(),
			p1.x(),
			p0.y(),
			p1.y(),
			p0.z(),
			mat.clone(),
		)));
		sides.add(Arc::new(XYRect::new(
			p0.x(),
			p1.x(),
			p0.y(),
			p1.y(),
			p1.z(),
			mat.clone(),
		)));

		sides.add(Arc::new(XZRect::new(
			p0.x(),
			p1.x(),
			p0.z(),
			p1.z(),
			p0.y(),
			mat.clone(),
		)));
		sides.add(Arc::new(XZRect::new(
			p0.x(),
			p1.x(),
			p0.z(),
			p1.z(),
			p1.y(),
			mat.clone(),
		)));

		sides.add(Arc::new(YZRect::new(
			p0.y(),
			p1.y(),
			p0.z(),
			p1.z(),
			p0.x(),
			mat.clone(),
		)));
		sides.add(Arc::new(YZRect::new(
			p0.y(),
			p1.y(),
			p0.z(),
			p1.z(),
			p1.x(),
			mat,
		)));

		Block {
			aabb: Aabb::new(p0, p1),
			sides,
		}
	}
}

impl Hittable for Block {
	fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
		Some(self.aabb)
	}

	fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
		self.sides.hit(r, t_min, t_max)
	}
}
