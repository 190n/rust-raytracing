use std::cmp::Ordering;

use rand::Rng;

use crate::ray::Ray;
use crate::util::degrees_to_radians;
use crate::vec::{Point3, Vec3};

pub struct Camera {
	origin: Point3,
	lower_left_corner: Point3,
	horizontal: Vec3,
	vertical: Vec3,
	u: Vec3,
	v: Vec3,
	w: Vec3,
	lens_radius: f64,
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
			w,
			lens_radius: aperture / 2.0,
		}
	}

	pub fn get_ray<R: Rng + ?Sized>(&self, rng: &mut R, s: f64, t: f64) -> Ray {
		let rd = self.lens_radius * Vec3::random_in_unit_disk(rng);
		let offset = self.u * rd.x() + self.v * rd.y();
		Ray::new(
			self.origin + offset,
			self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
		)
	}
}
