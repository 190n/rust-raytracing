use std::sync::Arc;

use crate::aabb::Aabb;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec::{Point3, Vec3};

#[derive(Debug)]
pub struct HitRecord {
	pub p: Point3,
	pub normal: Vec3,
	pub mat_ptr: Arc<dyn Material>,
	pub t: f64,
	pub front_face: bool,
}

impl HitRecord {
	pub fn set_face_normal(&mut self, r: Ray, outward_normal: Vec3) {
		self.front_face = Vec3::dot(r.direction(), outward_normal) < 0.0;
		self.normal = if self.front_face {
			outward_normal
		} else {
			-outward_normal
		};
	}
}

pub trait Hittable: Sync + Send {
	fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
	fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
}
