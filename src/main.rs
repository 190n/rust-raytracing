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
use material::{Dielectric, Lambertian, Material, Metal};
use ray::Ray;
use sphere::Sphere;
use vec::{Color, Point3, Vec3};

fn random_scene<R: Rng + ?Sized>(rng: &mut R) -> HittableList {
	let mut world = HittableList::new();

	let ground_material = Rc::new(Lambertian {
		albedo: Color::new(0.5, 0.5, 0.5),
	});
	world.add(Rc::new(Sphere::new(
		Point3::new(0.0, -1000.0, 0.0),
		1000.0,
		ground_material,
	)));

	for a in -11..11 {
		for b in -11..11 {
			let choose_mat = rng.gen::<f64>();
			let center = Point3::new(
				a as f64 + 0.9 * rng.gen::<f64>(),
				0.2,
				b as f64 + 0.9 * rng.gen::<f64>(),
			);

			if (center - Point3::new(4.0, 0.2, 0.0)).length_squared() > 0.81 {
				let sphere_material: Rc<dyn Material>;
				if choose_mat < 0.8 {
					let albedo = Color::random(rng) * Color::random(rng);
					sphere_material = Rc::new(Lambertian { albedo });
				} else if choose_mat < 0.95 {
					let albedo = Color::random_range(rng, 0.5, 1.0);
					let fuzz = rng.gen_range(0.0..0.5);
					sphere_material = Rc::new(Metal { albedo, fuzz });
				} else {
					sphere_material = Rc::new(Dielectric { ir: 1.5 });
				}
				world.add(Rc::new(Sphere::new(center, 0.2, sphere_material)));
			}
		}
	}

	let material1 = Rc::new(Dielectric { ir: 1.5 });
	world.add(Rc::new(Sphere::new(
		Point3::new(0.0, 1.0, 0.0),
		1.0,
		material1,
	)));
	let material2 = Rc::new(Lambertian {
		albedo: Color::new(0.4, 0.2, 0.1),
	});
	world.add(Rc::new(Sphere::new(
		Point3::new(-4.0, 1.0, 0.0),
		1.0,
		material2,
	)));
	let material3 = Rc::new(Metal {
		albedo: Color::new(0.7, 0.6, 0.5),
		fuzz: 0.0,
	});
	world.add(Rc::new(Sphere::new(
		Point3::new(4.0, 1.0, 0.0),
		1.0,
		material3,
	)));

	world
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

fn main() -> std::io::Result<()> {
	let aspect_ratio = 3.0 / 2.0;
	let image_width = 300;
	let image_height = (image_width as f64 / aspect_ratio) as i32;
	let samples_per_pixel = 50;
	let max_depth = 50;

	let mut rng = rand::thread_rng();

	let world = random_scene(&mut rng);

	let from = Point3::new(13.0, 2.0, 3.0);
	let at = Point3::zero();
	let dist = 10.0;
	let aperture = 0.1;

	let cam = Camera::new(
		from,
		at,
		Vec3::new(0.0, 1.0, 0.0),
		20.0,
		aspect_ratio,
		aperture,
		dist,
	);

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
