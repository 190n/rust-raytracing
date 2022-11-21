use rand::RngCore;

use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec::{Color, Vec3};

pub struct ScatterResult {
	pub attenuation: Color,
	pub scattered: Ray,
}

pub trait Material: std::fmt::Debug {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult>;
}

#[derive(Debug)]
pub struct Lambertian {
	pub albedo: Color,
}

impl Material for Lambertian {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
		let mut scatter_direction = rec.normal + Vec3::random_unit_vector(rng);
		if scatter_direction.near_zero() {
			scatter_direction = rec.normal;
		}

		Some(ScatterResult {
			scattered: Ray::new(rec.p, scatter_direction),
			attenuation: self.albedo,
		})
	}
}

#[derive(Debug)]
pub struct Metal {
	pub albedo: Color,
}

impl Material for Metal {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
		let reflected = r_in.direction().unit_vector().reflect(rec.normal);
		let scattered = Ray::new(rec.p, reflected);
		if scattered.direction().dot(rec.normal) > 0.0 {
			Some(ScatterResult {
				attenuation: self.albedo,
				scattered,
			})
		} else {
			None
		}
	}
}
