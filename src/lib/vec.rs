use std::{
	fmt::Display,
	iter::Sum,
	ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign},
};

use rand::Rng;

#[derive(Default, Clone, Copy, Debug)]
pub struct Vec3 {
	e: [f64; 3],
}

impl Vec3 {
	pub fn new(e0: f64, e1: f64, e2: f64) -> Self {
		Self { e: [e0, e1, e2] }
	}

	pub fn x(self) -> f64 {
		self.e[0]
	}

	pub fn y(self) -> f64 {
		self.e[1]
	}

	pub fn z(self) -> f64 {
		self.e[2]
	}

	pub fn length(self) -> f64 {
		self.length_squared().sqrt()
	}

	pub fn length_squared(self) -> f64 {
		self.e[0] * self.e[0] + self.e[1] * self.e[1] + self.e[2] * self.e[2]
	}

	pub fn dot(self, rhs: Self) -> f64 {
		self.e[0] * rhs.e[0] + self.e[1] * rhs.e[1] + self.e[2] * rhs.e[2]
	}

	pub fn cross(self, rhs: Self) -> Self {
		Self::new(
			self.e[1] * rhs.e[2] - self.e[2] * rhs.e[1],
			self.e[2] * rhs.e[0] - self.e[0] * rhs.e[2],
			self.e[0] * rhs.e[1] - self.e[1] * rhs.e[0],
		)
	}

	pub fn unit_vector(self) -> Self {
		self / self.length()
	}

	pub fn zero() -> Self {
		Self::default()
	}

	pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
		Self::new(rng.gen(), rng.gen(), rng.gen())
	}

	pub fn random_range<R: Rng + ?Sized>(rng: &mut R, min: f64, max: f64) -> Self {
		Self::new(
			rng.gen_range(min..max),
			rng.gen_range(min..max),
			rng.gen_range(min..max),
		)
	}

	pub fn random_in_unit_sphere<R: Rng + ?Sized>(rng: &mut R) -> Self {
		loop {
			let v = Self::random_range(rng, -1.0, 1.0);
			if v.length_squared() >= 1.0 {
				continue;
			}
			return v;
		}
	}

	pub fn random_unit_vector<R: Rng + ?Sized>(rng: &mut R) -> Self {
		Self::random_in_unit_sphere(rng).unit_vector()
	}

	pub fn random_in_hemisphere<R: Rng + ?Sized>(rng: &mut R, normal: Self) -> Self {
		let in_unit_sphere = Self::random_in_unit_sphere(rng);
		if Self::dot(in_unit_sphere, normal) > 0.0 {
			// same hemisphere
			in_unit_sphere
		} else {
			-in_unit_sphere
		}
	}

	pub fn near_zero(&self) -> bool {
		let epsilon = 1e-8;
		return self.e.iter().all(|c| c.abs() < epsilon);
	}

	pub fn reflect(self, n: Self) -> Self {
		self - 2.0 * self.dot(n) * n
	}

	pub fn refract(self, n: Self, etai_over_etat: f64) -> Self {
		let cos_theta = f64::min(Vec3::dot(-self, n), 1.0);
		let r_out_perp = etai_over_etat * (self + cos_theta * n);
		let r_out_parallel = -f64::sqrt(f64::abs(1.0 - r_out_perp.length_squared())) * n;
		r_out_perp + r_out_parallel
	}

	pub fn random_in_unit_disk<R: Rng + ?Sized>(rng: &mut R) -> Self {
		loop {
			let v = Self::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
			if v.length_squared() >= 1.0 {
				continue;
			}
			return v;
		}
	}

	pub fn min(&self, v: Vec3) -> Self {
		Self::new(self[0].min(v[0]), self[1].min(v[1]), self[2].min(v[2]))
	}

	pub fn max(&self, v: Vec3) -> Self {
		Self::new(self[0].max(v[0]), self[1].max(v[1]), self[2].max(v[2]))
	}
}

impl Neg for Vec3 {
	type Output = Self;
	fn neg(self) -> Self::Output {
		Self::new(-self.e[0], -self.e[1], -self.e[2])
	}
}

impl Index<usize> for Vec3 {
	type Output = f64;
	fn index(&self, index: usize) -> &Self::Output {
		&self.e[index]
	}
}

impl IndexMut<usize> for Vec3 {
	fn index_mut(&mut self, index: usize) -> &mut f64 {
		&mut self.e[index]
	}
}

impl AddAssign<Vec3> for Vec3 {
	fn add_assign(&mut self, rhs: Self) {
		self.e[0] += rhs.e[0];
		self.e[1] += rhs.e[1];
		self.e[2] += rhs.e[2];
	}
}

impl Add<Vec3> for Vec3 {
	type Output = Self;
	fn add(self, rhs: Self) -> Self::Output {
		let mut output = self;
		output += rhs;
		output
	}
}

impl SubAssign<Vec3> for Vec3 {
	fn sub_assign(&mut self, rhs: Self) {
		self.e[0] -= rhs.e[0];
		self.e[1] -= rhs.e[1];
		self.e[2] -= rhs.e[2];
	}
}

impl Sub<Vec3> for Vec3 {
	type Output = Self;
	fn sub(self, rhs: Self) -> Self::Output {
		let mut output = self;
		output -= rhs;
		output
	}
}

impl MulAssign<f64> for Vec3 {
	fn mul_assign(&mut self, rhs: f64) {
		self.e[0] *= rhs;
		self.e[1] *= rhs;
		self.e[2] *= rhs;
	}
}

impl Mul<f64> for Vec3 {
	type Output = Self;
	fn mul(self, rhs: f64) -> Self::Output {
		let mut output = self;
		output *= rhs;
		output
	}
}

impl Mul<Vec3> for f64 {
	type Output = Vec3;
	fn mul(self, rhs: Vec3) -> Self::Output {
		rhs * self
	}
}

impl MulAssign<Vec3> for Vec3 {
	fn mul_assign(&mut self, rhs: Self) {
		self.e[0] *= rhs.e[0];
		self.e[1] *= rhs.e[1];
		self.e[2] *= rhs.e[2];
	}
}

impl Mul<Vec3> for Vec3 {
	type Output = Self;
	fn mul(self, rhs: Self) -> Self::Output {
		let mut output = self;
		output *= rhs;
		output
	}
}

impl DivAssign<f64> for Vec3 {
	fn div_assign(&mut self, rhs: f64) {
		*self *= 1.0 / rhs;
	}
}

impl Div<f64> for Vec3 {
	type Output = Self;
	fn div(self, rhs: f64) -> Self::Output {
		let mut output = self;
		output /= rhs;
		output
	}
}

impl Sum for Vec3 {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.reduce(|a, b| a + b).unwrap_or_default()
	}
}

impl Display for Vec3 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
	}
}

pub type Point3 = Vec3;
