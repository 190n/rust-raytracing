use std::cmp::Ordering;
use std::sync::Arc;

use rand::{Rng, RngCore};

use crate::lib::{Color, Point3, Ray, Vec3};
use crate::object::{material::ScatterResult, HitRecord, Hittable, Material};
use crate::scene::Aabb;

#[derive(Debug)]
pub struct DebugMaterial(pub Color);

impl Material for DebugMaterial {
	fn scatter(
		&self,
		_rng: &mut dyn RngCore,
		_r_in: &Ray,
		_rec: &HitRecord,
	) -> Option<ScatterResult> {
		None
	}

	fn emitted(&self, _u: f64, _v: f64, _p: Point3) -> Color {
		self.0
	}
}

#[derive(Debug)]
pub enum BvhConstructionError {
	NoBoundingBox,
}

fn box_compare(
	a: &dyn Hittable,
	b: &dyn Hittable,
	axis: usize,
) -> Result<Ordering, BvhConstructionError> {
	let box_a = a.bounding_box(0.0, 0.0);
	let box_b = b.bounding_box(0.0, 0.0);
	if let (Some(a), Some(b)) = (box_a, box_b) {
		Ok(f64::total_cmp(&a.min()[axis], &b.min()[axis]))
	} else {
		Err(BvhConstructionError::NoBoundingBox)
	}
}

fn box_x_compare(a: &dyn Hittable, b: &dyn Hittable) -> Result<Ordering, BvhConstructionError> {
	box_compare(a, b, 0)
}

fn box_y_compare(a: &dyn Hittable, b: &dyn Hittable) -> Result<Ordering, BvhConstructionError> {
	box_compare(a, b, 1)
}

fn box_z_compare(a: &dyn Hittable, b: &dyn Hittable) -> Result<Ordering, BvhConstructionError> {
	box_compare(a, b, 2)
}

#[derive(Debug)]
pub struct BvhNode {
	left: Arc<dyn Hittable>,
	right: Arc<dyn Hittable>,
	bbox: Aabb,
	material: DebugMaterial,
}

impl BvhNode {
	pub fn new<R: Rng + ?Sized>(
		rng: &mut R,
		src_objects: &[Arc<dyn Hittable>],
		time0: f64,
		time1: f64,
	) -> Result<BvhNode, BvhConstructionError> {
		let (left, right) = match src_objects.len() {
			1 => (src_objects[0].clone(), src_objects[0].clone()),
			2 => (src_objects[0].clone(), src_objects[1].clone()),
			_ => {
				// convert objects into mutable array
				let mut objects: Vec<Arc<dyn Hittable>> =
					src_objects.iter().map(|p| p.clone()).collect();

				let comparator = [box_x_compare, box_y_compare, box_z_compare][rng.gen_range(0..3)];
				let mut errored = false;
				objects.sort_by(|a, b| match comparator(a.as_ref(), b.as_ref()) {
					Ok(ord) => ord,
					Err(_) => {
						errored = true;
						Ordering::Equal
					},
				});
				if errored {
					return Err(BvhConstructionError::NoBoundingBox);
				}

				let midpoint = src_objects.len() / 2;
				(
					Arc::new(BvhNode::new(rng, &objects[..midpoint], time0, time1)?)
						as Arc<dyn Hittable>,
					Arc::new(BvhNode::new(rng, &objects[midpoint..], time0, time1)?)
						as Arc<dyn Hittable>,
				)
			},
		};

		let box_left = left.bounding_box(time0, time1);
		let box_right = right.bounding_box(time0, time1);
		if let (Some(box_left), Some(box_right)) = (box_left, box_right) {
			let bbox = Aabb::surrounding_box(box_left, box_right);
			Ok(BvhNode {
				left,
				right,
				bbox,
				material: DebugMaterial(Color::random(rng).saturate()),
			})
		} else {
			Err(BvhConstructionError::NoBoundingBox)
		}
	}

	fn child_is_bvh(&self, child: &dyn Hittable) -> bool {
		std::ptr::metadata(self as &dyn Hittable) == std::ptr::metadata(child)
	}
}

impl Hittable for BvhNode {
	fn hit(&self, rng: &mut dyn RngCore, r: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
		if !self.bbox.hit(r, t_min, t_max) {
			None
		} else {
			if r.debug_bvh()
				&& rng.gen::<f64>() < 0.2
				&& !(self.child_is_bvh(self.left.as_ref())
					&& self.child_is_bvh(self.right.as_ref()))
			{
				return Some(HitRecord {
					p: Point3::zero(),
					normal: Vec3::zero(),
					mat_ptr: &self.material,
					t: t_min,
					u: 0.0,
					v: 0.0,
					front_face: true,
				});
			} else {
				// check if left and right are the same node; if so, we don't need to check them both
				if Arc::ptr_eq(&self.left, &self.right) {
					return self.left.hit(rng, r, t_min, t_max);
				}

				let hit_left = self.left.hit(rng, r, t_min, t_max);
				let hit_right = self.right.hit(
					rng,
					r,
					t_min,
					if let Some(ref rec) = hit_left {
						rec.t
					} else {
						t_max
					},
				);

				if hit_right.is_none() {
					hit_left
				} else {
					hit_right
				}
			}
		}
	}

	fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
		Some(self.bbox)
	}
}
