use crate::ray::Ray;
use crate::vec::Point3;

struct Aabb {
	minimum: Point3,
	maximum: Point3,
}

impl Aabb {
	pub fn new(minimum: Point3, maximum: Point3) -> Aabb {
		Aabb { minimum, maximum }
	}

	pub fn min(&self) -> Point3 {
		self.minimum
	}

	pub fn max(&self) -> Point3 {
		self.maximum
	}

	pub fn hit(&self, r: Ray, mut t_min: f64, mut t_max: f64) -> bool {
		for a in 0..3 {
			let t0 = f64::min(
				(self.minimum[a] - r.origin()[a]) / r.direction()[a],
				(self.maximum[a] - r.origin()[a]) / r.direction()[a],
			);
			let t1 = f64::max(
				(self.minimum[a] - r.origin()[a]) / r.direction()[a],
				(self.maximum[a] - r.origin()[a]) / r.direction()[a],
			);
			t_min = f64::max(t0, t_min);
			t_max = f64::min(t1, t_max);
			if t_max <= t_min {
				return false;
			}
		}
		true
	}
}
