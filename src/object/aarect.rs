use std::sync::Arc;

use rand::RngCore;

use super::{HitRecord, Hittable, Material};
use crate::lib::{Point3, Ray, Vec3};
use crate::scene::Aabb;

#[derive(Debug)]
pub struct XYRect {
	mat_ptr: Arc<dyn Material>,
	x0: f64,
	x1: f64,
	y0: f64,
	y1: f64,
	k: f64,
}

impl XYRect {
	pub fn new(x0: f64, x1: f64, y0: f64, y1: f64, k: f64, mat_ptr: Arc<dyn Material>) -> XYRect {
		XYRect {
			mat_ptr,
			x0,
			x1,
			y0,
			y1,
			k,
		}
	}
}

impl Hittable for XYRect {
	fn hit<'a>(
		&'a self,
		_rng: &mut dyn RngCore,
		r: Ray,
		t_min: f64,
		t_max: f64,
	) -> Option<HitRecord<'a>> {
		let t = (self.k - r.origin().z()) / r.direction().z();
		if t < t_min || t > t_max {
			return None;
		}

		let x = r.origin().x() + t * r.direction().x();
		let y = r.origin().y() + t * r.direction().y();
		if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
			return None;
		}

		let mut rec = HitRecord {
			u: (x - self.x0) / (self.x1 - self.x0),
			v: (y - self.y0) / (self.y1 - self.y0),
			t,
			mat_ptr: self.mat_ptr.as_ref(),
			p: r.at(t),
			normal: Vec3::zero(),
			front_face: false,
		};
		rec.set_face_normal(r, Vec3::new(0.0, 0.0, 1.0));
		Some(rec)
	}

	fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
		Some(Aabb::new(
			Point3::new(self.x0, self.y0, self.k - 0.0001),
			Point3::new(self.x1, self.y1, self.k + 0.0001),
		))
	}
}

#[derive(Debug)]
pub struct XZRect {
	mat_ptr: Arc<dyn Material>,
	x0: f64,
	x1: f64,
	z0: f64,
	z1: f64,
	k: f64,
}

impl XZRect {
	pub fn new(x0: f64, x1: f64, z0: f64, z1: f64, k: f64, mat_ptr: Arc<dyn Material>) -> XZRect {
		XZRect {
			mat_ptr,
			x0,
			x1,
			z0,
			z1,
			k,
		}
	}
}

impl Hittable for XZRect {
	fn hit<'a>(
		&'a self,
		_rng: &mut dyn RngCore,
		r: Ray,
		t_min: f64,
		t_max: f64,
	) -> Option<HitRecord<'a>> {
		let t = (self.k - r.origin().y()) / r.direction().y();
		if t < t_min || t > t_max {
			return None;
		}

		let x = r.origin().x() + t * r.direction().x();
		let z = r.origin().z() + t * r.direction().z();
		if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
			return None;
		}

		let mut rec = HitRecord {
			u: (x - self.x0) / (self.x1 - self.x0),
			v: (z - self.z0) / (self.z1 - self.z0),
			t,
			mat_ptr: self.mat_ptr.as_ref(),
			p: r.at(t),
			normal: Vec3::zero(),
			front_face: false,
		};
		rec.set_face_normal(r, Vec3::new(0.0, 1.0, 0.0));
		Some(rec)
	}

	fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
		Some(Aabb::new(
			Point3::new(self.x0, self.k - 0.0001, self.z0),
			Point3::new(self.x1, self.k + 0.0001, self.z1),
		))
	}
}

#[derive(Debug)]
pub struct YZRect {
	mat_ptr: Arc<dyn Material>,
	y0: f64,
	y1: f64,
	z0: f64,
	z1: f64,
	k: f64,
}

impl YZRect {
	pub fn new(y0: f64, y1: f64, z0: f64, z1: f64, k: f64, mat_ptr: Arc<dyn Material>) -> YZRect {
		YZRect {
			mat_ptr,
			y0,
			y1,
			z0,
			z1,
			k,
		}
	}
}

impl Hittable for YZRect {
	fn hit<'a>(
		&'a self,
		_rng: &mut dyn RngCore,
		r: Ray,
		t_min: f64,
		t_max: f64,
	) -> Option<HitRecord<'a>> {
		let t = (self.k - r.origin().x()) / r.direction().x();
		if t < t_min || t > t_max {
			return None;
		}

		let y = r.origin().y() + t * r.direction().y();
		let z = r.origin().z() + t * r.direction().z();
		if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
			return None;
		}

		let mut rec = HitRecord {
			u: (y - self.y0) / (self.y1 - self.y0),
			v: (z - self.z0) / (self.z1 - self.z0),
			t,
			mat_ptr: self.mat_ptr.as_ref(),
			p: r.at(t),
			normal: Vec3::zero(),
			front_face: false,
		};
		rec.set_face_normal(r, Vec3::new(1.0, 0.0, 0.0));
		Some(rec)
	}

	fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
		Some(Aabb::new(
			Point3::new(self.k - 0.0001, self.y0, self.z0),
			Point3::new(self.k + 0.0001, self.y1, self.z1),
		))
	}
}
