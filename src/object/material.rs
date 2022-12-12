use std::fmt::Debug;
use std::sync::Arc;

use rand::{Rng, RngCore};

use super::texture::SolidColor;
use super::HitRecord;
use super::Texture;
use crate::lib::{Color, Point3, Ray, Vec3};

pub struct ScatterResult {
	pub attenuation: Color,
	pub scattered: Ray,
}

pub trait Material: Debug + Sync + Send {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult>;
	fn emitted(&self, u: f64, v: f64, p: Point3) -> Color {
		// mark as unused without underscores in the signature
		(u, v, p);
		Color::zero()
	}
}

#[derive(Debug)]
pub struct Lambertian {
	albedo: Arc<dyn Texture>,
}

impl Lambertian {
	pub fn new(albedo: Arc<dyn Texture>) -> Lambertian {
		Lambertian { albedo }
	}

	pub fn with_color(color: Color) -> Lambertian {
		Lambertian {
			albedo: Arc::new(SolidColor::new(color)),
		}
	}
}

impl Material for Lambertian {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
		let mut scatter_direction = rec.normal + Vec3::random_unit_vector(rng);
		if scatter_direction.near_zero() {
			scatter_direction = rec.normal;
		}

		Some(ScatterResult {
			scattered: Ray::new(rec.p, scatter_direction, r_in.time()),
			attenuation: self.albedo.value(rec.u, rec.v, rec.p),
		})
	}
}

#[derive(Debug)]
pub struct Metal {
	albedo: Arc<dyn Texture>,
	fuzz: f64,
}

impl Metal {
	pub fn new(albedo: Arc<dyn Texture>, fuzz: f64) -> Metal {
		Metal { albedo, fuzz }
	}

	pub fn with_color(color: Color, fuzz: f64) -> Metal {
		Metal::new(Arc::new(SolidColor::new(color)), fuzz)
	}
}

impl Material for Metal {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
		let reflected = r_in.direction().unit_vector().reflect(rec.normal);
		let scattered = Ray::new(
			rec.p,
			reflected + self.fuzz * Vec3::random_in_unit_sphere(rng),
			r_in.time(),
		);
		if scattered.direction().dot(rec.normal) > 0.0 {
			Some(ScatterResult {
				attenuation: self.albedo.value(rec.u, rec.v, rec.p),
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
			scattered: Ray::new(rec.p, direction, r_in.time()),
		})
	}
}

#[derive(Debug)]
pub struct DiffuseLight {
	emit: Arc<dyn Texture>,
}

impl DiffuseLight {
	pub fn new(emit: Arc<dyn Texture>) -> DiffuseLight {
		DiffuseLight { emit }
	}

	pub fn with_color(color: Color) -> DiffuseLight {
		DiffuseLight {
			emit: Arc::new(SolidColor::new(color)),
		}
	}
}

impl Material for DiffuseLight {
	fn scatter(
		&self,
		_rng: &mut dyn RngCore,
		_r_in: &Ray,
		_rec: &HitRecord,
	) -> Option<ScatterResult> {
		None
	}

	fn emitted(&self, u: f64, v: f64, p: Point3) -> Color {
		self.emit.value(u, v, p)
	}
}

#[derive(Debug)]
pub struct Isotropic {
	albedo: Arc<dyn Texture>,
}

impl Isotropic {
	pub fn new(albedo: Arc<dyn Texture>) -> Isotropic {
		Isotropic { albedo }
	}
}

impl Material for Isotropic {
	fn scatter(&self, rng: &mut dyn RngCore, r_in: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
		Some(ScatterResult {
			attenuation: self.albedo.value(rec.u, rec.v, rec.p),
			scattered: Ray::new(rec.p, Vec3::random_in_unit_sphere(rng), r_in.time()),
		})
	}
}
