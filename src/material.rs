use rand::{Rng, RngCore};

use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec::{Color, Vec3};

pub struct ScatterResult {
	pub attenuation: Color,
	pub scattered: Ray,
}

pub trait Material: std::fmt::Debug + Sync + Send {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult>;
}

#[derive(Debug)]
pub struct Lambertian {
	pub albedo: Color,
}

impl Material for Lambertian {
	fn scatter(
		&self,
		rng: &mut dyn RngCore,
		_r_in: &Ray,
		rec: &HitRecord,
	) -> Option<ScatterResult> {
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

impl Dielectric {
	fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
		let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
		r0 = r0 * r0;
		r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
	}
}

impl Material for Dielectric {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
		let refraction_ratio = if rec.front_face {
			1.0 / self.ir
		} else {
			self.ir
		};

		let unit_direction = r_in.direction().unit_vector();
		let cos_theta = f64::min(Vec3::dot(-unit_direction, rec.normal), 1.0);
		let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

		let cannot_refract = refraction_ratio * sin_theta > 1.0;
		let direction = if cannot_refract
			|| Dielectric::reflectance(cos_theta, refraction_ratio) > rng.gen::<f64>()
		{
			unit_direction.reflect(rec.normal)
		} else {
			unit_direction.refract(rec.normal, refraction_ratio)
		};

		Some(ScatterResult {
			attenuation: Color::new(1.0, 1.0, 1.0),
			scattered: Ray::new(rec.p, direction),
		})
	}
}
