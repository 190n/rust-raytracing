use std::f64::consts::PI;

use rand::Rng;

use crate::lib::{Point3, Ray, Vec3};

#[derive(Clone, Copy)]
pub struct Camera {
	origin: Point3,
	lower_left_corner: Point3,
	horizontal: Vec3,
	vertical: Vec3,
	u: Vec3,
	v: Vec3,
	lens_radius: f64,
	time0: f64,
	time1: f64,
	aspect_ratio: f64,
}

fn degrees_to_radians(degrees: f64) -> f64 {
	degrees * PI / 180.0
}

impl Camera {
	/// vfov: vertical field of view in degrees
	pub fn new(
		look_from: Point3,
		look_at: Point3,
		vup: Vec3,
		vfov: f64,
		aspect_ratio: f64,
		aperture: f64,
		focus_dist: f64,
		time0: f64,
		time1: f64,
	) -> Self {
		let theta = degrees_to_radians(vfov);
		let h = f64::tan(theta / 2.0);
		let viewport_height = 2.0 * h;
		let viewport_width = aspect_ratio * viewport_height;

		let w = (look_from - look_at).unit_vector();
		let u = vup.cross(w).unit_vector();
		let v = w.cross(u);

		let origin = look_from;
		let horizontal = focus_dist * viewport_width * u;
		let vertical = focus_dist * viewport_height * v;
		let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

		Self {
			origin,
			horizontal,
			vertical,
			lower_left_corner,
			u,
			v,
			lens_radius: aperture / 2.0,
			time0,
			time1,
			aspect_ratio,
		}
	}

	pub fn get_ray<R: Rng + ?Sized>(&self, rng: &mut R, s: f64, t: f64) -> Ray {
		let rd = self.lens_radius * Vec3::random_in_unit_disk(rng);
		let offset = self.u * rd.x() + self.v * rd.y();
		Ray::new(
			self.origin + offset,
			self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
			rng.gen_range(self.time0..self.time1),
		)
	}

	pub fn aspect_ratio(&self) -> f64 {
		self.aspect_ratio
	}
}
