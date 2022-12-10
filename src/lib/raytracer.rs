use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use rand::Rng;

use crate::lib::{Color, Ray};
use crate::object::Hittable;
use crate::scene::Camera;

pub const TILE_SIZE: usize = 16;

pub struct Tile {
	pub pixels: [[Color; TILE_SIZE]; TILE_SIZE],
	pub x: usize,
	pub y: usize,
}

impl Tile {
	fn new(x: usize, y: usize) -> Self {
		Self {
			pixels: [[Color::zero(); TILE_SIZE]; TILE_SIZE],
			x,
			y,
		}
	}
}

fn ray_color(rng: &mut impl Rng, r: Ray, world: &dyn Hittable, depth: i32) -> Color {
	if depth <= 0 {
		return Color::zero();
	}

	if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
		if let Some(res) = rec.mat_ptr.scatter(rng, &r, &rec) {
			return res.attenuation * ray_color(rng, res.scattered, world, depth - 1);
		} else {
			return Color::zero();
		}
	}

	// background
	let unit_direction = r.direction().unit_vector();
	let t = 0.5 * (unit_direction.y() + 1.0);
	return (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
}

/// Render a scene
/// out:         queue to send completed tiles into
/// max_depth:   maximum number of light bounces per sample
/// current_pos: shared tracker of the next tile to render (top left corner)
pub fn render(
	out: mpsc::Sender<Tile>,
	rng: &mut impl Rng,
	world: Arc<dyn Hittable>,
	cam: Camera,
	(width, height): (usize, usize),
	samples_per_pixel: usize,
	max_depth: usize,
	current_pos: Arc<Mutex<(usize, usize)>>,
) -> (Duration, usize) {
	let mut total_time = Duration::ZERO;
	let mut total_pixels = 0usize;

	loop {
		let (x, y) = {
			let mut guard = current_pos.lock().unwrap();
			let previous_coords = *guard;
			guard.0 += TILE_SIZE;
			if guard.0 > width {
				guard.1 += TILE_SIZE;
				if guard.1 > height {
					return (total_time, total_pixels);
				}
				guard.0 = 0;
			}
			previous_coords
		};

		let mut tile = Tile::new(x, y);

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
					let r = cam.get_ray(rng, u, v);
					pixel_color += ray_color(rng, r, world.as_ref(), max_depth as i32);
				}
				tile.pixels[j - y][i - x] = pixel_color;
				total_pixels += 1;
			}
		}
		total_time += instant.elapsed();

		out.send(tile).unwrap();
	}
}
