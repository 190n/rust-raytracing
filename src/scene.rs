use std::sync::Arc;

use rand::Rng;

use crate::hittable_list::HittableList;
use crate::material::{Dielectric, Lambertian, Material, Metal};
use crate::sphere::Sphere;
use crate::vec::{Color, Point3};

pub fn random_scene<R: Rng + ?Sized>(rng: &mut R) -> HittableList {
	let mut world = HittableList::new();

	let ground_material = Arc::new(Lambertian {
		albedo: Color::new(0.5, 0.5, 0.5),
	});
	world.add(Arc::new(Sphere::new(
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
				let sphere_material: Arc<dyn Material + Sync + Send>;
				if choose_mat < 0.8 {
					let albedo = Color::random(rng) * Color::random(rng);
					sphere_material = Arc::new(Lambertian { albedo });
				} else if choose_mat < 0.95 {
					let albedo = Color::random_range(rng, 0.5, 1.0);
					let fuzz = rng.gen_range(0.0..0.5);
					sphere_material = Arc::new(Metal { albedo, fuzz });
				} else {
					sphere_material = Arc::new(Dielectric { ir: 1.5 });
				}
				world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
			}
		}
	}

	let material1 = Arc::new(Dielectric { ir: 1.5 });
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, 1.0, 0.0),
		1.0,
		material1,
	)));
	let material2 = Arc::new(Lambertian {
		albedo: Color::new(0.4, 0.2, 0.1),
	});
	world.add(Arc::new(Sphere::new(
		Point3::new(-4.0, 1.0, 0.0),
		1.0,
		material2,
	)));
	let material3 = Arc::new(Metal {
		albedo: Color::new(0.7, 0.6, 0.5),
		fuzz: 0.0,
	});
	world.add(Arc::new(Sphere::new(
		Point3::new(4.0, 1.0, 0.0),
		1.0,
		material3,
	)));

	world
}
