use std::sync::Arc;

use rand::RngCore;

use super::{HitRecord, Hittable, Material, Sphere};
use crate::common::{Point3, Ray, Vec3};
use crate::scene::Aabb;

#[derive(Debug)]
pub struct MovingSphere {
	center0: Point3,
	center1: Point3,
	time0: f64,
	time1: f64,
	radius: f64,
	mat_ptr: Arc<dyn Material>,
}

impl MovingSphere {
	pub fn new(
		center0: Point3,
		center1: Point3,
		time0: f64,
		time1: f64,
		radius: f64,
		mat_ptr: Arc<dyn Material>,
	) -> MovingSphere {
		MovingSphere {
			center0,
			center1,
			time0,
			time1,
			radius,
			mat_ptr,
		}
	}

	fn center(&self, time: f64) -> Point3 {
		self.center0
			+ ((time - self.time0) / (self.time1 - self.time0)) * (self.center1 - self.center0)
	}
}

impl Hittable for MovingSphere {
	fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
		let radius = self.radius.abs();
		let radius_vec = Vec3::new(radius, radius, radius);
		Some(Aabb::surrounding_box(
			Aabb::new(
				self.center(time0) - radius_vec,
				self.center(time0) + radius_vec,
			),
			Aabb::new(
				self.center(time1) - radius_vec,
				self.center(time1) + radius_vec,
			),
		))
	}

	fn hit<'a>(
		&'a self,
		_rng: &mut dyn RngCore,
		r: Ray,
		t_min: f64,
		t_max: f64,
	) -> Option<HitRecord<'a>> {
		return Sphere::hit_implementation(
			self.center(r.time()),
			self.radius,
			self.mat_ptr.as_ref(),
			r,
			t_min,
			t_max,
		);
	}
}
