use std::f64::consts::PI;
use std::sync::Arc;

use rand::RngCore;

use super::{HitRecord, Hittable, Material};
use crate::lib::{Point3, Ray, Vec3};
use crate::scene::Aabb;

#[derive(Debug)]
pub struct Sphere {
	center: Point3,
	radius: f64,
	mat_ptr: Arc<dyn Material>,
}

impl Sphere {
	fn get_sphere_uv(p: Point3) -> (f64, f64) {
		let theta = f64::acos(-p.y());
		let phi = f64::atan2(-p.z(), p.x()) + PI;
		let u = phi / (2.0 * PI);
		let v = theta / PI;
		(u, v)
	}

	pub fn new(center: Point3, radius: f64, mat_ptr: Arc<dyn Material>) -> Self {
		Self {
			center,
			radius,
			mat_ptr,
		}
	}

	pub fn hit_implementation<'a>(
		center: Point3,
		radius: f64,
		mat_ptr: &'a dyn Material,
		r: Ray,
		t_min: f64,
		t_max: f64,
	) -> Option<HitRecord<'a>> {
		let oc = r.origin() - center;
		let a = r.direction().length_squared();
		let half_b = Vec3::dot(oc, r.direction());
		let c = oc.length_squared() - radius * radius;

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
		let outward_normal = (p - center) / radius;
		let (u, v) = Sphere::get_sphere_uv(outward_normal);
		let mut hr = HitRecord {
			t: root,
			p,
			normal: Vec3::zero(),
			front_face: false,
			mat_ptr,
			u,
			v,
		};
		hr.set_face_normal(r, outward_normal);
		Some(hr)
	}
}

impl Hittable for Sphere {
	fn hit<'a>(
		&'a self,
		_rng: &mut dyn RngCore,
		r: Ray,
		t_min: f64,
		t_max: f64,
	) -> Option<HitRecord<'a>> {
		return Sphere::hit_implementation(
			self.center,
			self.radius,
			self.mat_ptr.as_ref(),
			r,
			t_min,
			t_max,
		);
	}

	fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
		let radius = self.radius.abs();
		Some(Aabb::new(
			self.center - Vec3::new(radius, radius, radius),
			self.center + Vec3::new(radius, radius, radius),
		))
	}
}
