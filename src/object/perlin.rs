use rand::Rng;

use crate::lib::{Point3, Vec3};

const POINT_COUNT: usize = 256;

#[derive(Debug, Clone)]
pub struct Perlin {
	vecs: Vec<Vec3>,
	perm_x: Vec<usize>,
	perm_y: Vec<usize>,
	perm_z: Vec<usize>,
}

impl Perlin {
	pub fn new<R: Rng + ?Sized>(rng: &mut R) -> Perlin {
		let mut vecs = vec![Vec3::zero(); POINT_COUNT];
		for i in 0..POINT_COUNT {
			vecs[i] = Vec3::random_range(rng, -1.0, 1.0).unit_vector();
		}

		Perlin {
			vecs,
			perm_x: Perlin::generate_perm(rng),
			perm_y: Perlin::generate_perm(rng),
			perm_z: Perlin::generate_perm(rng),
		}
	}

	pub fn noise(&self, p: Point3) -> f64 {
		let u = p.x() - p.x().floor();
		let v = p.y() - p.y().floor();
		let w = p.z() - p.z().floor();

		let i = p.x().floor() as isize;
		let j = p.y().floor() as isize;
		let k = p.z().floor() as isize;

		let mut c = [[[Vec3::zero(); 2]; 2]; 2];

		for di in 0..2isize {
			for dj in 0..2isize {
				for dk in 0..2isize {
					c[di as usize][dj as usize][dk as usize] = self.vecs[self.perm_x
						[(i + di).rem_euclid(POINT_COUNT as isize) as usize]
						^ self.perm_y[(j + dj).rem_euclid(POINT_COUNT as isize) as usize]
						^ self.perm_z[(k + dk).rem_euclid(POINT_COUNT as isize) as usize]];
				}
			}
		}
		Perlin::interp(c, u, v, w)
	}

	pub fn turbulence(&self, p: Point3, depth: usize) -> f64 {
		let mut acc = 0.0;
		let mut temp_p = p;
		let mut weight = 1.0;

		for _ in 0..depth {
			acc += weight * self.noise(temp_p);
			weight *= 0.5;
			temp_p *= 2.0;
		}

		acc.abs()
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

	fn interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
		let u = u * u * (3.0 - 2.0 * u);
		let v = v * v * (3.0 - 2.0 * v);
		let w = w * w * (3.0 - 2.0 * w);

		let mut acc = 0.0;
		for i in 0..2 {
			for j in 0..2 {
				for k in 0..2 {
					let (i_f, j_f, k_f) = (i as f64, j as f64, k as f64);
					let weight = Vec3::new(u - i_f, v - j_f, w - k_f);
					acc += (i_f * u + (1.0 - i_f) * (1.0 - u))
						* (j_f * v + (1.0 - j_f) * (1.0 - v))
						* (k_f * w + (1.0 - k_f) * (1.0 - w))
						* c[i][j][k].dot(weight);
				}
			}
		}
		acc
	}
}
