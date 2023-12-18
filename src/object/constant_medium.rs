use std::sync::Arc;

use rand::{Rng, RngCore};

use super::material::Isotropic;
use super::texture::SolidColor;
use super::{HitRecord, Hittable, Material, Sphere, Texture};
use crate::lib::{Color, Ray, Vec3};
use crate::scene::Aabb;

#[derive(Debug)]
pub struct ConstantMedium {
	boundary: Arc<dyn Hittable>,
	phase_function: Arc<dyn Material>,
	neg_inv_density: f64,
}

impl ConstantMedium {
	pub fn new(
		boundary: Arc<dyn Hittable>,
		density: f64,
		texture: Arc<dyn Texture>,
	) -> ConstantMedium {
		ConstantMedium {
			boundary,
			neg_inv_density: -1.0 / density,
			phase_function: Arc::new(Isotropic::new(texture)),
		}
	}

	pub fn with_color(boundary: Arc<dyn Hittable>, density: f64, color: Color) -> ConstantMedium {
		ConstantMedium {
			boundary,
			neg_inv_density: -1.0 / density,
			phase_function: Arc::new(Isotropic::new(Arc::new(SolidColor::new(color)))),
		}
	}
}

impl Hittable for ConstantMedium {
	fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
		self.boundary.bounding_box(time0, time1)
	}

	fn hit<'a>(
		&'a self,
		rng: &mut dyn RngCore,
		r: Ray,
		t_min: f64,
		t_max: f64,
	) -> Option<HitRecord<'a>> {
		if let Some(mut rec1) = self.boundary.hit(rng, r, f64::NEG_INFINITY, f64::INFINITY) {
			if let Some(mut rec2) = self.boundary.hit(rng, r, rec1.t + 0.0001, f64::INFINITY) {
				if rec1.t < t_min {
					rec1.t = t_min;
				}
				if rec2.t > t_max {
					rec2.t = t_max;
				}
				if rec1.t >= rec2.t {
					return None;
				}
				if rec1.t < 0.0 {
					rec1.t = 0.0;
				}

				let ray_length = r.direction().length();
				let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
				let hit_distance = self.neg_inv_density * rng.gen::<f64>().ln();

				if hit_distance > distance_inside_boundary {
					return None;
				}

				let t = rec1.t + hit_distance / ray_length;
				let p = r.at(t);
				let v = if let Some(bbox) = self.bounding_box(r.time(), r.time()) {
					let center = (bbox.min() + bbox.max()) / 2.0;
					Sphere::get_sphere_uv((p - center) / (bbox.max().x() - center.x())).1
				} else {
					1.0 // arbitrary
				};
				return Some(HitRecord {
					t,
					p: r.at(t),
					normal: Vec3::new(1.0, 0.0, 0.0), // arbitrary
					front_face: true,                 // arbitrary
					u: 1.0,                           // arbitrary
					v,
					mat_ptr: self.phase_function.as_ref(),
				});
			}
		}
		None
	}
}
