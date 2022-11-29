mod aabb;
mod args;
mod bvh;
mod camera;
mod color;
mod hittable;
mod hittable_list;
mod material;
mod ray;
mod raytracer;
mod scene;
mod sphere;
mod vec;

use std::io::{self, BufWriter, Write};
use std::{fs::File, sync::Arc, sync::Mutex, thread};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use args::WhichScene;
use bvh::BvhNode;
use color::write_color;
use raytracer::render;
use vec::Color;

fn main() -> io::Result<()> {
	let args = args::parse().unwrap_or_else(|e| {
		eprintln!("{}", e);
		args::show_help();
		std::process::exit(1);
	});

	let aspect_ratio = 3.0 / 2.0;
	let image_width = args.width;
	let image_height = (image_width as f64 / aspect_ratio) as usize;
	let samples_per_pixel = args.samples;
	let max_depth = args.depth;
	let num_threads = args.threads;

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

	let (world, cam) = match args.scene {
		WhichScene::Random => scene::random_scene(&mut rng),
		WhichScene::Figure19 => scene::figure19_scene(),
		WhichScene::Refraction => scene::refraction_scene(),
	};
	let world = Arc::new(
		BvhNode::new(&mut rng, world.as_ref(), 0.0, 0.0).unwrap_or_else(|e| {
			eprintln!("error constructing BVH: {:?}", e);
			std::process::exit(1);
		}),
	);

	let images = vec![Vec::<Color>::with_capacity(image_width * image_height); num_threads]
		.into_iter()
		.enumerate();
	let mut handles = Vec::<thread::JoinHandle<Vec<Color>>>::with_capacity(num_threads);
	let counter = Arc::new(Mutex::new(0usize));
	let mut logger_assigned = false;

	for (i, mut buf) in images {
		// pick a variable number of samples per thread so that we total to the number of samples
		// configured. i.e. with 4 threads and 25 samples, samples will be [6, 6, 6, 7].
		let samples =
			(i + 1) * samples_per_pixel / num_threads - i * samples_per_pixel / num_threads;
		let w = world.clone();
		let mut thread_rng = Xoshiro256PlusPlus::from_seed(rng.gen());
		let c = counter.clone();
		// ceiling division, to ensure that the logger thread is one of the threads that processes
		// more samples
		let samples_for_logger = (samples_per_pixel / num_threads)
			+ (if samples_per_pixel % num_threads == 0 {
				0
			} else {
				1
			});
		let will_log = !logger_assigned && samples == samples_for_logger;
		if will_log {
			logger_assigned = true;
		}
		handles.push(thread::spawn(move || {
			render(
				&mut buf,
				&mut thread_rng,
				w,
				cam,
				(image_width, image_height),
				samples,
				max_depth,
				if will_log {
					Some(num_threads * image_height)
				} else {
					None
				},
				c,
			);
			buf
		}));
	}

	let output: Vec<Vec<Color>> = handles.into_iter().map(|h| h.join().unwrap()).collect();

	write!(buffered, "P6\n{} {}\n255\n", image_width, image_height)?;

	// for each pixel, sum every thread's samples
	for i in 0..(image_width * image_height) {
		let color = output.iter().map(|v| v[i]).sum();
		write_color(&mut buffered, color, samples_per_pixel)?;
	}

	buffered.flush()?;
	Ok(())
}
