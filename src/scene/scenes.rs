use std::sync::Arc;

use rand::Rng;

use super::Camera;
use super::HittableList;
use crate::lib::{Color, Point3, Vec3};
use crate::object::material::{Dielectric, Lambertian, Material, Metal};
use crate::object::texture::NoiseTexture;
use crate::object::texture::SolidColor;
use crate::object::texture::{CheckerTexture, StripeTexture};
use crate::object::{MovingSphere, Sphere};

pub type Scene = (f64, HittableList, Camera);

fn standard_camera() -> Camera {
	let from = Point3::new(13.0, 2.0, 3.0);
	let at = Point3::zero();
	let dist = 10.0;
	let aperture = 0.1;
	let aspect = 3.0 / 2.0;

	Camera::new(
		from,
		at,
		Vec3::new(0.0, 1.0, 0.0),
		20.0,
		aspect,
		aperture,
		dist,
		0.0,
		1.0,
	)
}

pub fn figure19_scene() -> Scene {
	let mut world = HittableList::new();
	let glass = Arc::new(Dielectric {
		ir: 1.5,
		color: Color::new(1.0, 1.0, 1.0),
	});

	// ground
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, -100.5, -1.0),
		100.0,
		Arc::new(Lambertian::with_color(Color::new(0.8, 0.8, 0.0))),
	)));
	// center
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, 0.0, -1.0),
		0.5,
		Arc::new(Lambertian::with_color(Color::new(0.1, 0.2, 0.5))),
	)));
	// left (outer)
	world.add(Arc::new(Sphere::new(
		Point3::new(-1.0, 0.0, -1.0),
		0.5,
		glass.clone(),
	)));
	// left (inner)
	world.add(Arc::new(Sphere::new(
		Point3::new(-1.0, 0.0, -1.0),
		-0.45,
		glass,
	)));
	// right
	world.add(Arc::new(Sphere::new(
		Point3::new(1.0, 0.0, -1.0),
		0.5,
		Arc::new(Metal {
			albedo: Color::new(0.8, 0.6, 0.2),
			fuzz: 0.0,
		}),
	)));

	let aspect = 16.0 / 9.0;

	let cam = Camera::new(
		Point3::new(-2.0, 2.0, 1.0),
		Point3::new(0.0, 0.0, -1.0),
		Vec3::new(0.0, 1.0, 0.0),
		20.0,
		aspect,
		0.0,
		1.0,
		0.0,
		1.0,
	);
	(aspect, world, cam)
}

pub fn refraction_scene() -> Scene {
	let mut world = HittableList::new();
	let ball_material = Arc::new(Lambertian::with_color(Color::new(0.0, 0.6, 0.0)));
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, -1000.0, 0.0),
		1000.0,
		Arc::new(Dielectric {
			ir: 1.3,
			color: Color::new(0.75, 0.75, 1.0),
		}),
	)));

	for i in -5..=5 {
		world.add(Arc::new(Sphere::new(
			Point3::zero() + Vec3::new(0.0, 0.5 * i as f64, -2.0 * i as f64),
			1.0,
			ball_material.clone(),
		)));
	}

	let aspect = 2.0;

	let cam = Camera::new(
		Point3::new(-15.0, 3.0, 0.0),
		Point3::zero(),
		Vec3::new(0.0, 1.0, 0.0),
		60.0,
		aspect,
		0.0,
		1.0,
		0.0,
		1.0,
	);
	(aspect, world, cam)
}

pub fn random_scene<R: Rng + ?Sized>(rng: &mut R, next_week: bool, gay: bool) -> Scene {
	let mut world = HittableList::new();

	let ground_material = if next_week {
		Arc::new(Lambertian::new(Arc::new(CheckerTexture::with_colors(
			Color::new(0.2, 0.3, 0.1),
			Color::new(0.9, 0.9, 0.9),
		))))
	} else {
		Arc::new(Lambertian::with_color(Color::new(0.5, 0.5, 0.5)))
	};
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, -1000.0, 0.0),
		1000.0,
		ground_material,
	)));

	let trans = Arc::new(Lambertian::new(StripeTexture::trans_sphere()));
	let rainbow = Arc::new(Lambertian::new(StripeTexture::rainbow_sphere()));
	let enby = Arc::new(Lambertian::new(StripeTexture::enby_sphere()));

	for a in -11..11 {
		for b in -11..11 {
			let choose_mat = rng.gen::<f64>();
			let center = Point3::new(
				a as f64 + 0.9 * rng.gen::<f64>(),
				0.2,
				b as f64 + 0.9 * rng.gen::<f64>(),
			);

			if (center - Point3::new(4.0, 0.2, 0.0)).length_squared() > 0.81 {
				let sphere_material: Arc<dyn Material> = if gay {
					if choose_mat < 1.0 / 3.0 {
						trans.clone()
					} else if choose_mat < 2.0 / 3.0 {
						rainbow.clone()
					} else {
						enby.clone()
					}
				} else {
					if choose_mat < 0.8 {
						Arc::new(Lambertian::with_color(
							Color::random(rng) * Color::random(rng),
						))
					} else if choose_mat < 0.95 {
						Arc::new(Metal {
							albedo: Color::random_range(rng, 0.5, 1.0),
							fuzz: rng.gen_range(0.0..0.5),
						})
					} else {
						Arc::new(Dielectric {
							ir: 1.5,
							color: Color::random_range(rng, 0.5, 1.0),
						})
					}
				};

				if next_week && choose_mat < 0.8 {
					let center2 = center + Vec3::new(0.0, rng.gen_range(0.0..0.5), 0.0);
					world.add(Arc::new(MovingSphere::new(
						center,
						center2,
						0.0,
						1.0,
						0.2,
						sphere_material,
					)))
				} else {
					world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
				}
			}
		}
	}

	let material1 = Arc::new(Dielectric {
		ir: 1.5,
		color: Color::new(1.0, 1.0, 1.0),
	});
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, 1.0, 0.0),
		1.0,
		material1,
	)));
	let material2 = Arc::new(Lambertian::with_color(Color::new(0.4, 0.2, 0.1)));
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

	(1.5, world, standard_camera())
}

pub fn perlin_spheres<R: Rng + ?Sized>(rng: &mut R) -> Scene {
	let mut objects = HittableList::new();
	let white = Arc::new(SolidColor::new(Color::new(1.0, 1.0, 1.0)));
	let black = Arc::new(SolidColor::new(Color::zero()));
	let perlin = Arc::new(NoiseTexture::new(rng, white, black));
	let material = Arc::new(Lambertian::new(perlin));

	objects.add(Arc::new(Sphere::new(
		Point3::new(0.0, -1000.0, 0.0),
		1000.0,
		material.clone(),
	)));
	objects.add(Arc::new(Sphere::new(
		Point3::new(0.0, 2.0, 0.0),
		2.0,
		material.clone(),
	)));
	(1.5, objects, standard_camera())
}
