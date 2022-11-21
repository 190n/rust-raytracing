mod camera;
mod color;
mod hittable;
mod hittable_list;
mod material;
mod ray;
mod sphere;
mod util;
mod vec;

use std::{io::BufWriter, io::Write, rc::Rc};

use rand::Rng;

use camera::Camera;
use color::write_color;
use hittable::Hittable;
use hittable_list::HittableList;
use material::{Dielectric, Lambertian, Metal};
use ray::Ray;
use sphere::Sphere;
use vec::{Color, Point3, Vec3};

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

fn main() -> std::io::Result<()> {
	let aspect_ratio = 16.0 / 9.0;
	let image_width = 400;
	let image_height = (image_width as f64 / aspect_ratio) as i32;
	let samples_per_pixel = 100;
	let max_depth = 50;

	let mut world = HittableList::new();

	let material_ground = Rc::new(Lambertian {
		albedo: Color::new(0.8, 0.8, 0.0),
	});
	let material_center = Rc::new(Lambertian {
		albedo: Color::new(0.1, 0.2, 0.5),
	});
	let material_left = Rc::new(Dielectric { ir: 1.5 });
	let material_right = Rc::new(Metal {
		albedo: Color::new(0.8, 0.6, 0.2),
		fuzz: 0.0,
	});

	world.add(Rc::new(Sphere::new(
		Point3::new(0.0, -100.5, -1.0),
		100.0,
		material_ground,
	)));
	world.add(Rc::new(Sphere::new(
		Point3::new(0.0, 0.0, -1.0),
		0.5,
		material_center,
	)));
	world.add(Rc::new(Sphere::new(
		Point3::new(-1.0, 0.0, -1.0),
		0.5,
		material_left.clone(),
	)));
	world.add(Rc::new(Sphere::new(
		Point3::new(-1.0, 0.0, -1.0),
		-0.4,
		material_left,
	)));
	world.add(Rc::new(Sphere::new(
		Point3::new(1.0, 0.0, -1.0),
		0.5,
		material_right,
	)));

	let from = Point3::new(3.0, 3.0, 2.0);
	let at = Point3::new(0.0, 0.0, -1.0);
	let dist = (at - from).length();

	let cam = Camera::new(
		from,
		at,
		Vec3::new(0.0, 1.0, 0.0),
		20.0,
		aspect_ratio,
		2.0,
		dist,
	);

	let mut rng = rand::thread_rng();

	print!("P6\n{} {}\n255\n", image_width, image_height);
	let mut buffered = BufWriter::new(std::io::stdout());

	for j in (0..image_height).rev() {
		eprint!("\rscanlines remaining: {} ", j);
		for i in 0..image_width {
			let mut pixel_color = Color::zero();
			for _ in 0..samples_per_pixel {
				let u = (i as f64 + rng.gen::<f64>()) / (image_width - 1) as f64;
				let v = (j as f64 + rng.gen::<f64>()) / (image_height - 1) as f64;
				let r = cam.get_ray(&mut rng, u, v);
				pixel_color += ray_color(&mut rng, r, &world, max_depth);
			}
			write_color(&mut buffered, pixel_color, samples_per_pixel)?;
		}
	}

	buffered.flush()?;
	Ok(())
}
