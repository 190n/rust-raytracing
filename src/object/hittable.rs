use std::fmt::{Debug, Display};
use std::sync::Arc;

use super::Material;
use crate::lib::{Point3, Ray, Vec3};
use crate::scene::Aabb;

#[derive(Debug)]
pub struct HitRecord {
	pub p: Point3,
	pub normal: Vec3,
	pub mat_ptr: Arc<dyn Material>,
	pub t: f64,
	pub u: f64,
	pub v: f64,
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

pub trait Hittable: Sync + Send + Debug {
	fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
	fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
}

#[derive(Debug)]
pub struct Translate {
	child: Arc<dyn Hittable>,
	offset: Vec3,
}

impl Translate {
	pub fn new(child: Arc<dyn Hittable>, offset: Vec3) -> Translate {
		Translate { child, offset }
	}
}

impl Hittable for Translate {
	fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
		self.child
			.bounding_box(time0, time1)
			.map(|bb| Aabb::new(bb.min() + self.offset, bb.max() + self.offset))
	}

	fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
		let translated_ray = Ray::new(r.origin() - self.offset, r.direction(), r.time());
		self.child.hit(translated_ray, t_min, t_max).map(|mut rec| {
			rec.p += self.offset;
			rec.set_face_normal(translated_ray, rec.normal);
			rec
		})
	}
}

#[derive(Debug)]
pub struct RotateY {
	child: Arc<dyn Hittable>,
	sin_theta: f64,
	cos_theta: f64,
}

impl RotateY {
	pub fn new(child: Arc<dyn Hittable>, angle: f64) -> RotateY {
		let radians = angle.to_radians();
		let sin_theta = radians.sin();
		let cos_theta = radians.cos();

		RotateY {
			child,
			sin_theta,
			cos_theta,
		}
	}
}

impl Hittable for RotateY {
	fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
		self.child.bounding_box(time0, time1).map(|bbox| {
			let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
			let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

			for i in 0..2 {
				for j in 0..2 {
					for k in 0..2 {
						let (i_f, j_f, k_f) = (i as f64, j as f64, k as f64);
						let x = i_f * bbox.max().x() + (1.0 - i_f) * bbox.min().x();
						let y = j_f * bbox.max().y() + (1.0 - j_f) * bbox.min().y();
						let z = k_f * bbox.max().z() + (1.0 - k_f) * bbox.min().z();

						let new_x = self.cos_theta * x + self.sin_theta * z;
						let new_z = -self.sin_theta * x + self.cos_theta * z;

						let tester = Vec3::new(new_x, y, new_z);
						min = min.min(tester);
						max = max.max(tester);
					}
				}
			}

			Aabb::new(min, max)
		})
	}

	fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
		let mut origin = r.origin();
		let mut direction = r.direction();

		origin[0] = self.cos_theta * r.origin()[0] - self.sin_theta * r.origin()[2];
		origin[2] = self.sin_theta * r.origin()[0] + self.cos_theta * r.origin()[2];

		direction[0] = self.cos_theta * r.direction()[0] - self.sin_theta * r.direction()[2];
		direction[2] = self.sin_theta * r.direction()[0] + self.cos_theta * r.direction()[2];

		let rotated_ray = Ray::new(origin, direction, r.time());
		self.child.hit(rotated_ray, t_min, t_max).map(|mut rec| {
			let mut p = rec.p;
			let mut normal = rec.normal;

			p[0] = self.cos_theta * rec.p[0] + self.sin_theta * rec.p[2];
			p[2] = -self.sin_theta * rec.p[0] + self.cos_theta * rec.p[2];

			normal[0] = self.cos_theta * rec.normal[0] + self.sin_theta * rec.normal[2];
			normal[2] = -self.sin_theta * rec.normal[0] + self.cos_theta * rec.normal[2];

			rec.p = p;
			rec.set_face_normal(rotated_ray, normal);
			rec
		})
	}
}
