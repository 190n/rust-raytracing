use std::rc::Rc;

use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec::{Point3, Vec3};

#[derive(Debug)]
pub struct Sphere {
	center: Point3,
	radius: f64,
	mat_ptr: Rc<dyn Material>,
}

impl Sphere {
	pub fn new(center: Point3, radius: f64, mat_ptr: Rc<dyn Material>) -> Self {
		Self {
			center,
			radius,
			mat_ptr,
		}
	}
}

impl Hittable for Sphere {
	fn hit(&self, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
		let oc = r.origin() - self.center;
		let a = r.direction().length_squared();
		let half_b = Vec3::dot(oc, r.direction());
		let c = oc.length_squared() - self.radius * self.radius;

		let discriminant = half_b * half_b - a * c;
		if discriminant < 0.0 {
			return None;
		}
		let sqrtd = discriminant.sqrt();

		let mut root = (-half_b - sqrtd) / a;
		if root < t_min || t_max < root {
			root = (-half_b + sqrtd) / a;
			if root < t_min || t_max < root {
				return None;
			}
		}

		let p = r.at(root);
		let outward_normal = (p - self.center) / self.radius;
		let mut hr = HitRecord {
			t: root,
			p,
			normal: Vec3::zero(),
			front_face: false,
			mat_ptr: self.mat_ptr.clone(),
		};
		hr.set_face_normal(r, outward_normal);
		Some(hr)
	}
}
