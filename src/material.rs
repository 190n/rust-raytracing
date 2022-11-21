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
	/// metal fuzziness, 0-1
	pub fuzz: f64,
}

impl Material for Metal {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
		let reflected = r_in.direction().unit_vector().reflect(rec.normal);
		let scattered = Ray::new(
			rec.p,
			reflected + self.fuzz * Vec3::random_in_unit_sphere(rng),
		);
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

#[derive(Debug)]
pub struct Dielectric {
	/// index of refraction
	pub ir: f64,
}

impl Material for Dielectric {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
		let refraction_ratio = if rec.front_face {
			1.0 / self.ir
		} else {
			self.ir
		};

		let unit_direction = r_in.direction().unit_vector();
		let refracted = unit_direction.refract(rec.normal, refraction_ratio);
		Some(ScatterResult {
			attenuation: Color::new(1.0, 1.0, 1.0),
			scattered: Ray::new(rec.p, refracted),
		})
	}
}
