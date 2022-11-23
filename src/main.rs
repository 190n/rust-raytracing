mod args;
mod camera;
mod color;
mod hittable;
mod hittable_list;
mod material;
mod ray;
mod raytracer;
mod scene;
mod sphere;
mod util;
mod vec;

use std::{fs::File, io::BufWriter, io::Write, sync::Arc, sync::Mutex, thread};

use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use camera::Camera;
use color::write_color;
use raytracer::render;
use scene::random_scene;
use vec::{Color, Point3, Vec3};

fn main() -> std::io::Result<()> {
	let a = args::parse().unwrap_or_else(|e| {
		eprintln!("{}", e);
		args::show_help();
		std::process::exit(1);
	});

	let aspect_ratio = 3.0 / 2.0;
	let image_width = a.width;
	let image_height = (image_width as f64 / aspect_ratio) as usize;
	let samples_per_pixel = a.samples;
	let max_depth = a.depth;
	let num_threads = a.threads;

	// use 64 bits of seed; rest are zeroed
	let mut seed = [0u8; 32];
	for (i, &b) in a.seed.to_le_bytes().iter().enumerate() {
		seed[i] = b;
	}
	let mut rng = Xoshiro256PlusPlus::from_seed(seed);

	let mut buffered: BufWriter<Box<dyn std::io::Write>> = if let Some(filename) = a.output {
		BufWriter::new(Box::new(File::create(filename)?))
	} else {
		BufWriter::new(Box::new(std::io::stdout()))
	};

	let world = Arc::new(random_scene(&mut rng));

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

	let images =
		vec![Vec::<Color>::with_capacity(image_width * image_height); num_threads as usize]
			.into_iter()
			.enumerate();
	let mut handles = Vec::<thread::JoinHandle<Vec<Color>>>::with_capacity(num_threads as usize);
	let counter = Arc::new(Mutex::new(0usize));
	let mut logger_assigned = false;

	for (i, mut buf) in images {
		// pick a variable number of samples per thread so that we total to the number of samples
		// configured. i.e. with 4 threads and 25 samples, samples will be [6, 6, 6, 7].
		let samples =
			(i + 1) * samples_per_pixel / num_threads - i * samples_per_pixel / num_threads;
		let w = world.clone();
		let mut r = rng.clone();
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
				&mut r,
				w,
				cam,
				(image_width, image_height),
				samples,
				max_depth,
				if will_log {
					Some(num_threads as usize * image_height)
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

	for i in 0..(image_width * image_height) {
		let color = output.iter().map(|v| v[i]).sum();
		write_color(&mut buffered, color, samples_per_pixel)?;
	}

	buffered.flush()?;
	Ok(())
}
