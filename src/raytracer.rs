use std::sync::{Arc, Mutex};

use rand::Rng;

use crate::camera::Camera;
use crate::hittable::Hittable;
use crate::ray::Ray;
use crate::vec::Color;

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
/// v:         vector where colors should be stored
/// max_depth: maximum number of light bounces per sample
/// log:       Some(usize) = this thread should log, and the usize is height * num_threads
///            None        = no logging from this thread
/// counter:   shared counter of how many scanlines have been rendered so far; initialize as 0
pub fn render(
	v: &mut Vec<Color>,
	rng: &mut impl Rng,
	world: Arc<dyn Hittable>,
	cam: Camera,
	(width, height): (usize, usize),
	samples_per_pixel: usize,
	max_depth: usize,
	log: Option<usize>,
	counter: Arc<Mutex<usize>>,
) -> () {
	for j in (0..height).rev() {
		{
			let mut c = counter.lock().unwrap();
			*c += 1;
		}
		if let Some(total_lines) = log {
			let c = counter.lock().unwrap().clone();
			eprint!(
				"\rprogress: {:5>.2}%",
				c as f64 / total_lines as f64 * 100.0
			);
		}
		for i in 0..width {
			let mut pixel_color = Color::zero();
			for _ in 0..samples_per_pixel {
				let u = (i as f64 + rng.gen::<f64>()) / (width - 1) as f64;
				let v = (j as f64 + rng.gen::<f64>()) / (height - 1) as f64;
				let r = cam.get_ray(rng, u, v);
				pixel_color += ray_color(rng, r, world.as_ref(), max_depth as i32);
			}
			v.push(pixel_color);
		}
	}

	if log.is_some() {
		eprint!("\n");
	}
}
