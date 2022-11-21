use crate::vec::{Point3, Vec3};

#[derive(Default, Copy, Clone)]
pub struct Ray {
	orig: Point3,
	dir: Vec3,
}

impl Ray {
	pub fn new(orig: Point3, dir: Vec3) -> Self {
		Self { orig, dir }
	}

	pub fn origin(self) -> Point3 {
		self.orig
	}

	pub fn direction(self) -> Vec3 {
		self.dir
	}

	pub fn at(self, t: f64) -> Point3 {
		self.orig + t * self.dir
	}
}
