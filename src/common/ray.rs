use crate::common::{Point3, Vec3};

#[derive(Default, Copy, Clone)]
pub struct Ray {
	orig: Point3,
	dir: Vec3,
	tm: f64,
	debug_bvh: bool,
}

impl Ray {
	pub fn new(orig: Point3, dir: Vec3, tm: f64, debug_bvh: bool) -> Self {
		Self {
			orig,
			dir,
			tm,
			debug_bvh: debug_bvh,
		}
	}

	pub fn origin(&self) -> Point3 {
		self.orig
	}

	pub fn direction(&self) -> Vec3 {
		self.dir
	}

	pub fn time(&self) -> f64 {
		self.tm
	}

	pub fn debug_bvh(&self) -> bool {
		self.debug_bvh
	}

	pub fn at(&self, t: f64) -> Point3 {
		self.orig + t * self.dir
	}
}
