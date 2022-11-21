use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vec::{Point3, Vec3};

#[derive(Default, Debug)]
pub struct Sphere {
	center: Point3,
	radius: f64,
}

impl Sphere {
	pub fn new(center: Point3, radius: f64) -> Self {
		Self { center, radius }
	}

	pub fn center(&self) -> Point3 {
		self.center
	}

	pub fn radius(&self) -> f64 {
		self.radius
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
			..Default::default()
		};
		hr.set_face_normal(r, outward_normal);
		Some(hr)
	}
}
