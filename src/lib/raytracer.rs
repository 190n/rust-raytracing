use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::lib::{Color, Ray};
use crate::object::Hittable;
use crate::scene::Camera;

use super::color::OutputColor;

pub const TILE_SIZE: usize = 16;

pub struct Tile {
	pub pixels: [[OutputColor; TILE_SIZE]; TILE_SIZE],
	pub x: usize,
	pub y: usize,
}

impl Tile {
	fn new(x: usize, y: usize) -> Self {
		Self {
			pixels: [[OutputColor(0, 0, 0); TILE_SIZE]; TILE_SIZE],
			x,
			y,
		}
	}
}

fn ray_color(
	rng: &mut impl Rng,
	r: Ray,
	background: Color,
	world: &dyn Hittable,
	depth: i32,
) -> Color {
	if depth <= 0 {
		return Color::zero();
	}

	if let Some(rec) = world.hit(rng, r, 0.001, f64::INFINITY) {
		let emitted = rec.mat_ptr.emitted(rec.u, rec.v, rec.p);
		if let Some(res) = rec.mat_ptr.scatter(rng, &r, &rec) {
			emitted + res.attenuation * ray_color(rng, res.scattered, background, world, depth - 1)
		} else {
			emitted
		}
	} else {
		background
	}
}

/// Render a scene
/// out:         queue to send completed tiles into
/// max_depth:   maximum number of light bounces per sample
/// current_pos: shared tracker of the next tile to render (top left corner)
pub fn render(
	out: mpsc::Sender<Tile>,
	seed: u64,
	world: Arc<dyn Hittable>,
	cam: Camera,
	background: Color,
	(width, height): (usize, usize),
	samples_per_pixel: usize,
	max_depth: usize,
	current_pos: Arc<Mutex<(usize, usize)>>,
) -> (Duration, usize) {
	let mut total_time = Duration::ZERO;
	let mut total_pixels = 0usize;
	let mut done = false;

	while !done {
		let (x, y) = {
			let mut guard = current_pos.lock().unwrap();
			let previous_coords = *guard;
			guard.0 += TILE_SIZE;
			if guard.0 >= width {
				guard.0 = 0;
				guard.1 += TILE_SIZE;
			}
			if guard.1 >= height {
				done = true;
				// in this case, another thread already rendered the last tile (putting guard at
				// (0, max height)) so we just need to return
				if guard.0 == TILE_SIZE {
					guard.0 = 0;
					return (total_time, total_pixels);
				}
			}

			previous_coords
		};

		let mut tile = Tile::new(x, y);
		let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed ^ x as u64 ^ y as u64);

		let instant = Instant::now();
		for j in (y..(y + TILE_SIZE)).rev() {
			if j >= height {
				continue;
			}

			for i in x..(x + TILE_SIZE) {
				if i >= width {
					continue;
				}

				let mut pixel_color = Color::zero();
				for _ in 0..samples_per_pixel {
					let u = (i as f64 + rng.gen::<f64>()) / (width - 1) as f64;
					let v = (j as f64 + rng.gen::<f64>()) / (height - 1) as f64;
					let r = cam.get_ray(&mut rng, u, v);
					pixel_color +=
						ray_color(&mut rng, r, background, world.as_ref(), max_depth as i32);
				}
				let factor = 1.0 / samples_per_pixel as f64;
				tile.pixels[j - y][i - x] = (pixel_color * factor).tonemap();
				total_pixels += 1;
			}
		}
		total_time += instant.elapsed();

		out.send(tile).unwrap();
	}

	return (total_time, total_pixels);
}
