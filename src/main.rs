mod lib;
mod object;
mod scene;

use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use lib::args::{self, WhichScene};
use lib::color::write_color;
use lib::raytracer::{render, Tile, TILE_SIZE};
use lib::Color;
use scene::{scenes, BvhNode};

struct RayRate(f64);

impl Display for RayRate {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let (measurement, prefix) = if self.0 >= 1e9 {
			(self.0 / 1e9, "G")
		} else if self.0 >= 1e6 {
			(self.0 / 1e6, "M")
		} else if self.0 >= 1e3 {
			(self.0 / 1e3, "k")
		} else {
			(self.0, "")
		};
		write!(f, "{:6.2} {}Ray/s", measurement, prefix)?;
		Ok(())
	}
}

struct Eta(Duration);

impl Display for Eta {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let total_secs = self.0.as_secs();
		let hours = total_secs / 3600;
		let mins = (total_secs % 3600) / 60;
		let secs = total_secs % 60;
		if hours > 0 {
			write!(f, "{}:{:02}:{:02}", hours, mins, secs)?;
		} else if mins > 0 {
			write!(f, "{}:{:02}", mins, secs)?;
		} else {
			write!(f, "{}", secs)?;
		}
		Ok(())
	}
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

	let (world, cam, background) = match args.scene {
		WhichScene::Weekend => scenes::random_scene(&mut rng, false, false),
		WhichScene::Gay => scenes::random_scene(&mut rng, false, true),
		WhichScene::Tuesday => scenes::random_scene(&mut rng, true, false),
		WhichScene::Figure19 => scenes::figure19_scene(),
		WhichScene::Refraction => scenes::refraction_scene(),
		WhichScene::Perlin => scenes::perlin_spheres(&mut rng),
		WhichScene::Earth => scenes::earth().expect("failed to load texture"),
	};
	let world = Arc::new(
		BvhNode::new(&mut rng, world.as_ref(), 0.0, 1.0).unwrap_or_else(|e| {
			eprintln!("error constructing BVH: {:?}", e);
			std::process::exit(1);
		}),
	);

	let aspect_ratio = cam.aspect_ratio();
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
				background,
				(image_width, image_height),
				samples_per_pixel,
				max_depth,
				pos,
			)
		}));
	}

	drop(send);

	let mut pixels_so_far = 0;
	let start_time = Instant::now();

	while let Ok(tile) = recv.recv() {
		for i in tile.y..(tile.y + TILE_SIZE) {
			if i >= image_height {
				continue;
			}

			let width = usize::min(TILE_SIZE, image_width - tile.x);
			let final_x = tile.x + width;
			image[image_height - i - 1][tile.x..final_x]
				.copy_from_slice(&tile.pixels[i - tile.y][0..width]);
			pixels_so_far += width;
		}

		let progress = pixels_so_far as f64 / (image_width * image_height) as f64;
		let elapsed = start_time.elapsed();
		let remaining = (elapsed.div_f64(progress)) - elapsed;
		eprint!(
			"\rprogress: {:6.2}% | eta: {}s  ",
			progress * 100.0,
			Eta(remaining),
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
				eprintln!("thread {:3}: {}", i, RayRate(rays_sec));
				rays_sec
			})
			.sum();
		eprintln!("total:      {}", RayRate(total_rays_sec));
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
