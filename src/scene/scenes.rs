use std::sync::Arc;

use image::ImageResult;
use rand::Rng;

use super::BvhNode;
use super::Camera;
use super::HittableList;
use crate::lib::{Color, Point3, Vec3};
use crate::object::material::{Dielectric, DiffuseLight, Lambertian, Material, Metal};
use crate::object::texture::{
	CheckerTexture, FunctionTexture, ImageTexture, NoiseTexture, SolidColor, StripeTexture, Texture,
};
use crate::object::{
	Block, ConstantMedium, Hittable, MovingSphere, RotateY, Sphere, Translate, XYRect, XZRect,
	YZRect,
};

pub type Scene = (HittableList, Camera, Color);

fn sky() -> Color {
	Color::new(0.7, 0.8, 1.0)
}

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

	let gay_materials: [Arc<dyn Material>; 16] = std::array::from_fn(|i| {
		let texture = match i % 4 {
			0 => StripeTexture::trans_sphere(),
			1 => StripeTexture::rainbow_sphere(),
			2 => StripeTexture::enby_sphere(),
			3 => StripeTexture::bi_sphere(),
			4.. => unreachable!(),
		};

		return match i / 4 {
			0 => Arc::new(Lambertian::new(texture)) as Arc<dyn Material>,
			1 => Arc::new(Metal::new(texture, 0.2)) as Arc<dyn Material>,
			2 => Arc::new(Dielectric { ir: 1.5 }) as Arc<dyn Material>,
			3 => Arc::new(DiffuseLight::new(Arc::new(FunctionTexture({
				let texture_ref = texture.clone();
				move |u, v, p| {
					let value = texture_ref.value(u, v, p);
					return 20.0 * value;
				}
			})))) as Arc<dyn Material>,
			4.. => unreachable!(),
		};
	});

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
					gay_materials[(choose_mat * gay_materials.len() as f64) as usize].clone()
				} else {
					if choose_mat < 0.8 {
						Arc::new(Lambertian::with_color(
							Color::random(rng) * Color::random(rng),
						))
					} else if choose_mat < 0.95 {
						Arc::new(Metal::with_color(
							Color::random_range(rng, 0.5, 1.0),
							rng.gen_range(0.0..0.5),
						))
					} else {
						Arc::new(Dielectric { ir: 1.5 })
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

	let material1 = Arc::new(Dielectric { ir: 1.5 });
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, 1.0, 0.0),
		1.0,
		material1,
	)));
	let material2: Arc<dyn Material> = if gay {
		Arc::new(DiffuseLight::with_color(Color::new(4.0, 4.0, 4.0)))
	} else {
		Arc::new(Lambertian::with_color(Color::new(0.4, 0.2, 0.1)))
	};
	world.add(Arc::new(Sphere::new(
		Point3::new(-4.0, 1.0, 0.0),
		1.0,
		material2,
	)));
	let material3 = Arc::new(Metal::with_color(Color::new(0.7, 0.6, 0.5), 0.0));
	world.add(Arc::new(Sphere::new(
		Point3::new(4.0, 1.0, 0.0),
		1.0,
		material3,
	)));

	if gay {
		world.add(Arc::new(ConstantMedium::with_color(
			Arc::new(Sphere::new(
				Point3::zero(),
				25.0,
				Arc::new(Dielectric { ir: 0.0 }),
			)),
			0.05,
			Color::new(0.04, 0.08, 0.1),
		)));
	}

	(
		world,
		standard_camera(),
		if gay { Color::zero() } else { sky() },
	)
}

pub fn perlin_spheres<R: Rng + ?Sized>(rng: &mut R) -> Scene {
	let mut world = HittableList::new();
	let black = Arc::new(SolidColor::new(Color::zero()));
	let white = Arc::new(SolidColor::new(Color::new(1.0, 1.0, 1.0)));
	let perlin1 = Arc::new(NoiseTexture::new(rng, black.clone(), white.clone(), 4.0, 7));

	let noise = NoiseTexture::new(rng, black, white, 10.0, 50);
	let perlin2 = Arc::new(FunctionTexture(move |u, v, p| {
		let v = noise.value(u, v, p).x();
		let blue = Color::new(0.0, 0.1, 0.15);
		let white = Color::new(1.0, 1.0, 1.0);
		// interpolate, but darken a lot using the exponent
		blue + (white - blue) * v.powi(15)
	}));

	let material1 = Arc::new(Metal::new(perlin1, 0.3));
	let material2 = Arc::new(Lambertian::new(perlin2));

	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, -1000.0, 0.0),
		1000.0,
		material1,
	)));
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, 2.0, 0.0),
		2.0,
		material2,
	)));
	(
		world,
		Camera::new(
			Point3::new(13.0, 2.0, 3.0),
			Point3::zero(),
			Vec3::new(0.0, 1.0, 0.0),
			45.0,
			1.5,
			0.0,
			1.0,
			0.0,
			1.0,
		),
		sky(),
	)
}

pub fn earth() -> ImageResult<Scene> {
	let mut world = HittableList::new();
	let earth_texture = Arc::new(ImageTexture::new("textures/earthmap.jpg")?);
	let earth_mat = Arc::new(Lambertian::new(earth_texture));
	let globe = Arc::new(Sphere::new(Point3::zero(), 2.0, earth_mat));
	world.add(globe);
	Ok((
		world,
		Camera::new(
			Point3::new(14.0, 0.0, 0.0),
			Point3::zero(),
			Vec3::new(0.0, 1.0, 0.0),
			20.0,
			1.5,
			0.0,
			1.0,
			0.0,
			1.0,
		),
		sky(),
	))
}

pub fn cornell_box() -> Scene {
	let mut world = HittableList::new();

	let red = Arc::new(Lambertian::with_color(Color::new(0.65, 0.05, 0.05)));
	let white = Arc::new(Lambertian::with_color(Color::new(0.73, 0.73, 0.73)));
	let green = Arc::new(Lambertian::with_color(Color::new(0.12, 0.45, 0.15)));
	let light = Arc::new(DiffuseLight::with_color(Color::new(15.0, 15.0, 15.0)));

	world.add(Arc::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, green)));
	world.add(Arc::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, red)));
	world.add(Arc::new(XZRect::new(
		213.0, 343.0, 227.0, 332.0, 554.99, light,
	)));
	world.add(Arc::new(XZRect::new(
		0.0,
		555.0,
		0.0,
		555.0,
		0.0,
		white.clone(),
	)));
	world.add(Arc::new(XZRect::new(
		0.0,
		555.0,
		0.0,
		555.0,
		555.0,
		white.clone(),
	)));
	world.add(Arc::new(XYRect::new(
		0.0,
		555.0,
		0.0,
		555.0,
		555.0,
		white.clone(),
	)));

	let mut block1: Arc<dyn Hittable> = Arc::new(Block::new(
		Point3::zero(),
		Point3::new(165.0, 320.0, 165.0),
		white.clone(),
	));
	block1 = Arc::new(RotateY::new(block1, 15.0));
	block1 = Arc::new(Translate::new(block1, Vec3::new(265.0, 0.0, 295.0)));
	world.add(block1);

	let mut block2: Arc<dyn Hittable> = Arc::new(Block::new(
		Point3::zero(),
		Point3::new(165.0, 165.0, 165.0),
		white,
	));
	block2 = Arc::new(RotateY::new(block2, -18.0));
	block2 = Arc::new(Translate::new(block2, Vec3::new(130.0, 0.0, 65.0)));
	world.add(block2);

	let from = Point3::new(278.0, 278.0, -800.0);
	let to = Point3::new(278.0, 278.0, 0.0);

	(
		world,
		Camera::new(
			from,
			to,
			Vec3::new(0.0, 1.0, 0.0),
			40.0,
			1.0,
			0.1,
			(to - from).length(),
			0.0,
			1.0,
		),
		Color::zero(),
	)
}

pub fn bisexual_lighting() -> Scene {
	let (mut world, cam, background) = cornell_box();

	world.add(Arc::new(XZRect::new(
		0.0,
		555.0,
		0.0,
		555.0,
		554.9,
		Arc::new(DiffuseLight::new(Arc::new(
			StripeTexture::bi().as_ref().clone() * 2.0,
		))),
	)));
	world.add(Arc::new(Sphere::new(
		Point3::new(400.0, 80.0, 100.0),
		50.0,
		Arc::new(Dielectric { ir: 1.5 }),
	)));

	let mist = Arc::new(SolidColor::new(Color::new(0.0, 0.0, 0.5)));
	let bogus = Arc::new(Lambertian::with_color(Color::zero()));
	world.add(Arc::new(ConstantMedium::new(
		Arc::new(Block::new(
			Point3::new(100.0, 200.0, 0.0),
			Point3::new(200.0, 300.0, 400.0),
			bogus,
		)),
		0.01,
		mist,
	)));

	world.add(Arc::new(RotateY::new(
		Arc::new(Block::new(
			Point3::new(150.0, 50.0, 0.0),
			Point3::new(250.0, 300.0, 10.0),
			Arc::new(Dielectric { ir: 1.1 }),
		)),
		30.0,
	)));

	(world, cam, background)
}

pub fn week<R: Rng + ?Sized>(rng: &mut R) -> ImageResult<Scene> {
	let mut world = HittableList::new();
	let ground = Arc::new(Lambertian::with_color(Color::new(0.48, 0.83, 0.53)));

	for i in 0..20 {
		for j in 0..20 {
			let w = 100.0;
			let x0 = -1000.0 + i as f64 * w;
			let z0 = -1000.0 + j as f64 * w;
			let y0 = 0.0;
			let x1 = x0 + w;
			let y1 = rng.gen_range(1.0..101.0);
			let z1 = z0 + w;
			world.add(Arc::new(Block::new(
				Point3::new(x0, y0, z0),
				Point3::new(x1, y1, z1),
				ground.clone(),
			)));
		}
	}

	let light = Arc::new(DiffuseLight::with_color(Color::new(7.0, 7.0, 7.0)));
	world.add(Arc::new(XZRect::new(
		123.0, 423.0, 147.0, 412.0, 554.0, light,
	)));

	let center1 = Point3::new(400.0, 400.0, 200.0);
	let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
	let moving_sphere_mat = Arc::new(Lambertian::with_color(Color::new(0.7, 0.3, 0.1)));
	world.add(Arc::new(MovingSphere::new(
		center1,
		center2,
		0.0,
		1.0,
		50.0,
		moving_sphere_mat,
	)));

	world.add(Arc::new(Sphere::new(
		Point3::new(260.0, 150.0, 45.0),
		50.0,
		Arc::new(Dielectric { ir: 1.5 }),
	)));
	world.add(Arc::new(Sphere::new(
		Point3::new(0.0, 150.0, 145.0),
		50.0,
		Arc::new(Metal::with_color(Color::new(0.8, 0.8, 0.9), 1.0)),
	)));

	let boundary = Arc::new(Sphere::new(
		Point3::new(360.0, 150.0, 145.0),
		70.0,
		Arc::new(Dielectric { ir: 1.5 }),
	));
	world.add(boundary.clone());
	world.add(Arc::new(ConstantMedium::with_color(
		boundary,
		0.2,
		Color::new(0.2, 0.4, 0.9),
	)));
	let boundary2 = Arc::new(Sphere::new(
		Point3::zero(),
		5000.0,
		Arc::new(Dielectric { ir: f64::NAN }),
	));
	world.add(Arc::new(ConstantMedium::with_color(
		boundary2,
		0.0001,
		Color::new(1.0, 1.0, 1.0),
	)));

	let emat = Arc::new(Lambertian::new(Arc::new(ImageTexture::new(
		"textures/earthmap.jpg",
	)?)));
	world.add(Arc::new(Sphere::new(
		Point3::new(400.0, 200.0, 400.0),
		100.0,
		emat,
	)));
	let pertext = Arc::new(NoiseTexture::new(
		rng,
		Arc::new(SolidColor::new(Color::zero())),
		Arc::new(SolidColor::new(Color::new(1.0, 1.0, 1.0))),
		0.1,
		7,
	));
	world.add(Arc::new(Sphere::new(
		Point3::new(220.0, 280.0, 300.0),
		80.0,
		Arc::new(Lambertian::new(pertext)),
	)));

	let mut spheres = HittableList::new();
	let white = Arc::new(Lambertian::with_color(Color::new(0.73, 0.73, 0.73)));
	for i in 0..1000 {
		spheres.add(Arc::new(Sphere::new(
			Point3::random_range(rng, 0.0, 165.0),
			10.0,
			if i < 50 {
				Arc::new(DiffuseLight::with_color(Color::random_range(
					rng, 0.0, 50.0,
				)))
			} else {
				white.clone()
			},
		)));
	}
	world.add(Arc::new(Translate::new(
		Arc::new(RotateY::new(
			Arc::new(
				BvhNode::new(rng, spheres.as_ref(), 0.0, 1.0)
					.expect("spheres all have bounding boxes"),
			),
			15.0,
		)),
		Vec3::new(-100.0, 270.0, 395.0),
	)));

	let from = Point3::new(478.0, 278.0, -600.0);
	let at = Point3::new(278.0, 278.0, 0.0);
	Ok((
		world,
		Camera::new(
			from,
			at,
			Vec3::new(0.0, 1.0, 0.0),
			40.0,
			1.0,
			0.1,
			(at - from).length(),
			0.0,
			1.0,
		),
		Color::zero(),
	))
}
