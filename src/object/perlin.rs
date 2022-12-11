use rand::Rng;

use crate::lib::Point3;

const POINT_COUNT: usize = 256;

#[derive(Debug)]
pub struct Perlin {
	ranfloat: Vec<f64>,
	perm_x: Vec<usize>,
	perm_y: Vec<usize>,
	perm_z: Vec<usize>,
}

impl Perlin {
	pub fn new<R: Rng + ?Sized>(rng: &mut R) -> Perlin {
		let mut ranfloat = vec![0.0; POINT_COUNT];
		for i in 0..POINT_COUNT {
			ranfloat[i] = rng.gen();
		}

		Perlin {
			ranfloat,
			perm_x: Perlin::generate_perm(rng),
			perm_y: Perlin::generate_perm(rng),
			perm_z: Perlin::generate_perm(rng),
		}
	}

	pub fn noise(&self, p: Point3) -> f64 {
		let i = ((4.0 * p.x()) as isize).rem_euclid(POINT_COUNT as isize) as usize;
		let j = ((4.0 * p.y()) as isize).rem_euclid(POINT_COUNT as isize) as usize;
		let k = ((4.0 * p.z()) as isize).rem_euclid(POINT_COUNT as isize) as usize;

		self.ranfloat[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
	}

	fn generate_perm<R: Rng + ?Sized>(rng: &mut R) -> Vec<usize> {
		let mut p = vec![0usize; POINT_COUNT];
		for i in 0..POINT_COUNT {
			p[i] = i;
		}
		Perlin::permute(rng, &mut p);
		p
	}

	fn permute<R: Rng + ?Sized>(rng: &mut R, p: &mut [usize]) {
		for i in (1..p.len()).rev() {
			let target = rng.gen_range(0..i);
			p.swap(i, target);
		}
	}
}
