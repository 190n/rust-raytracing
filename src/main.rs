mod aabb;
mod args;
mod bvh;
mod camera;
mod color;
mod hittable;
mod hittable_list;
mod material;
mod moving_sphere;
mod ray;
mod raytracer;
mod scene;
mod sphere;
mod vec;

use std::io::{self, BufWriter, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::{fs::File, thread, thread::JoinHandle};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use args::WhichScene;
use bvh::BvhNode;
use color::write_color;
use raytracer::{render, Tile};
use vec::Color;

fn print_ray_rate(rate: f64) -> () {
	let (measurement, prefix) = if rate >= 1e9 {
		(rate / 1e9, "G")
	} else if rate >= 1e6 {
		(rate / 1e6, "M")
	} else if rate >= 1e3 {
		(rate / 1e3, "k")
	} else {
		(rate, "")
	};
	eprint!("{:6.2} {}Ray/s", measurement, prefix);
}

fn main() -> io::Result<()> {
	let args = args::parse().unwrap_or_else(|e| {
		eprintln!("{}", e);
		args::show_help();
		std::process::exit(1);
	});

	// use 64 bits of seed; rest are zeroed
	let mut seed = [0u8; 32];
	for (i, &b) in args.seed.to_le_bytes().iter().enumerate() {
		seed[i] = b;
	}
	let mut rng = Xoshiro256PlusPlus::from_seed(seed);

	let mut buffered: BufWriter<Box<dyn Write>> = if let Some(filename) = args.output {
		BufWriter::new(Box::new(File::create(filename)?))
	} else {
		BufWriter::new(Box::new(io::stdout()))
	};

	let (aspect_ratio, world, cam) = match args.scene {
		WhichScene::Random => scene::random_scene(&mut rng, false),
		WhichScene::RandomMoving => scene::random_scene(&mut rng, true),
		WhichScene::Figure19 => scene::figure19_scene(),
		WhichScene::Refraction => scene::refraction_scene(),
	};
	let world = Arc::new(
		BvhNode::new(&mut rng, world.as_ref(), 0.0, 1.0).unwrap_or_else(|e| {
			eprintln!("error constructing BVH: {:?}", e);
			std::process::exit(1);
		}),
	);

	let image_width = args.width;
	let image_height = (image_width as f64 / aspect_ratio) as usize;
	let samples_per_pixel = args.samples;
	let max_depth = args.depth;
	let num_threads = args.threads;

	let mut handles: Vec<JoinHandle<(Duration, usize)>> = Vec::with_capacity(num_threads);

	let mut image: Vec<Vec<Color>> = vec![vec![Color::zero(); image_width]; image_height];
	let (send, recv) = mpsc::channel::<Tile>();
	let current_pos = Arc::new(Mutex::new((0usize, 0usize)));

	for _ in 0..num_threads {
		let w = world.clone();
		let mut thread_rng = Xoshiro256PlusPlus::from_seed(rng.gen());
		let pos = current_pos.clone();
		let q = send.clone();
		handles.push(thread::spawn(move || {
			render(
				q,
				&mut thread_rng,
				w,
				cam,
				(image_width, image_height),
				samples_per_pixel,
				max_depth,
				pos,
			)
		}));
	}

	drop(send);

	let mut pixels_so_far = 0;

	while let Ok(tile) = recv.recv() {
		for i in tile.y..(tile.y + raytracer::TILE_SIZE) {
			if i >= image_height {
				continue;
			}

			let width = usize::min(raytracer::TILE_SIZE, image_width - tile.x);
			let final_x = tile.x + width;
			image[image_height - i - 1][tile.x..final_x]
				.copy_from_slice(&tile.pixels[i - tile.y][0..width]);
			pixels_so_far += width;
		}

		eprint!(
			"\rprogress: {:6.2}%",
			pixels_so_far as f64 / (image_width * image_height) as f64 * 100.0
		);
	}

	eprint!("\n");

	if args.verbose {
		let total_rays_sec: f64 = handles
			.into_iter()
			.map(|h| h.join().unwrap())
			.enumerate()
			.map(|(i, (duration, pixels))| {
				let rays = pixels * samples_per_pixel;
				let rays_sec = (rays as f64) / (duration.as_millis() as f64) * 1000.0;
				eprint!("thread {:3}: ", i);
				print_ray_rate(rays_sec);
				eprint!("\n");
				rays_sec
			})
			.sum();
		eprint!("total:      ");
		print_ray_rate(total_rays_sec);
		eprint!("\n");
	}

	write!(buffered, "P6\n{} {}\n255\n", image_width, image_height)?;

	for row in image {
		for pixel in row {
			write_color(&mut buffered, pixel, samples_per_pixel)?;
		}
	}

	buffered.flush()?;
	Ok(())
}
