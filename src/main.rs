mod camera;
mod color;
mod hittable;
mod hittable_list;
mod ray;
mod sphere;
mod util;
mod vec;

use std::rc::Rc;

use rand::Rng;

use camera::Camera;
use color::write_color;
use hittable::Hittable;
use ray::Ray;
use sphere::Sphere;
use vec::{Color, Point3};

use crate::hittable_list::HittableList;

fn ray_color(r: Ray, world: &dyn Hittable) -> Color {
	if let Some(rec) = world.hit(r, 0.0, f64::INFINITY) {
		return 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0));
	}
	let unit_direction = r.direction().unit_vector();
	let t = 0.5 * (unit_direction.y() + 1.0);
	return (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
}

fn main() -> std::io::Result<()> {
	let aspect_ratio = 16.0 / 9.0;
	let image_width = 400;
	let image_height = (image_width as f64 / aspect_ratio) as i32;
	let samples_per_pixel = 100;

	let mut world = HittableList::new();
	world.add(Rc::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
	world.add(Rc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

	let cam = Camera::new();

	let mut rng = rand::thread_rng();

	print!("P3\n{} {}\n255\n", image_width, image_height);

	for j in (0..image_height).rev() {
		eprint!("\rscanlines remaining: {} ", j);
		for i in 0..image_width {
			let mut pixel_color = Color::zero();
			for _ in 0..samples_per_pixel {
				let u = (i as f64 + rng.gen::<f64>()) / (image_width - 1) as f64;
				let v = (j as f64 + rng.gen::<f64>()) / (image_height - 1) as f64;
				let r = cam.get_ray(u, v);
				pixel_color += ray_color(r, &world);
			}
			write_color(&mut std::io::stdout(), pixel_color, samples_per_pixel)?;
		}
	}

	Ok(())
}
