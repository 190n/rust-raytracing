#![feature(ptr_metadata)]

mod common;
mod object;
mod output;
mod scene;

use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{self, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use exr::image::{write::WritableImage, Image};
use exr::image::{AnyChannel, AnyChannels, FlatSamples};
use exr::math::Vec2;
use exr::meta::attribute::Chromaticities;
use half::f16;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use smallvec::smallvec;
use time::OffsetDateTime;

use common::args::{self, FileFormat, WhichScene};
use common::raytracer::{render, Tile, TILE_SIZE};
use common::Color;
use output::png::PngRenderingIntent;
use output::{ImageWriter, PngWriter, PpmWriter};
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

struct CheckValidChannel<const CHANNEL: u8>;

impl<const CHANNEL: u8> CheckValidChannel<CHANNEL> {
	const OK: () = assert!(CHANNEL < 3);
}

// fake enum since rust doesn't support enums in const generics
mod channel {
	pub const RED: u8 = 0;
	pub const GREEN: u8 = 1;
	pub const BLUE: u8 = 2;
}

fn channel_from_image<const CHANNEL: u8>(image: &Vec<Vec<Color>>, bit_depth: u8) -> FlatSamples {
	let _ = CheckValidChannel::<CHANNEL>::OK;
	match bit_depth {
		16 => FlatSamples::F16(
			image
				.iter()
				.flat_map(|row| {
					row.iter().map(|pixel| {
						f16::from_f64(match CHANNEL {
							channel::RED => pixel.x(),
							channel::GREEN => pixel.y(),
							channel::BLUE => pixel.z(),
							_ => unreachable!(),
						})
					})
				})
				.collect(),
		),
		32 => FlatSamples::F32(
			image
				.iter()
				.flat_map(|row| {
					row.iter().map(|pixel| {
						(match CHANNEL {
							channel::RED => pixel.x(),
							channel::GREEN => pixel.y(),
							channel::BLUE => pixel.z(),
							_ => unreachable!(),
						}) as f32
					})
				})
				.collect(),
		),
		_ => unreachable!(),
	}
}

fn main() -> io::Result<()> {
	let args = args::parse().unwrap_or_else(|e| {
		eprintln!("{}", e);
		args::show_help();
		std::process::exit(1);
	});
	let mut world_rng = Xoshiro256PlusPlus::seed_from_u64(args.world_seed);

	let mut output: Box<dyn Write> = if let Some(filename) = args.output {
		Box::new(File::create(filename)?)
	} else {
		Box::new(io::stdout())
	};

	let (world, cam, background) = match args.scene {
		WhichScene::Weekend => scenes::random_scene(&mut world_rng, false, false),
		WhichScene::Gay => scenes::random_scene(&mut world_rng, false, true),
		WhichScene::Tuesday => scenes::random_scene(&mut world_rng, true, false),
		WhichScene::Perlin => scenes::perlin_spheres(&mut world_rng),
		WhichScene::Earth => scenes::earth().expect("failed to load texture"),
		WhichScene::Cornell => scenes::cornell_box(),
		WhichScene::Bisexual => scenes::bisexual_lighting(),
		WhichScene::Week => scenes::week(&mut world_rng).expect("failed to load texture"),
	};
	let world = Arc::new(
		BvhNode::new(&mut world_rng, world.as_ref(), 0.0, 1.0).unwrap_or_else(|e| {
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
	let current_pos = Arc::new(Mutex::new((0usize, 0usize)));

	// sender is scoped in this block so that the main thread's sender gets dropped
	// that way the channel is closed as soon as every worker thread has finished
	let recv = {
		let (send, recv) = mpsc::channel::<Tile>();
		for _ in 0..num_threads {
			let w = world.clone();
			let pos = current_pos.clone();
			let q = send.clone();
			handles.push(thread::spawn(move || {
				render(
					q,
					args.sample_seed,
					w,
					cam,
					background,
					(image_width, image_height),
					samples_per_pixel,
					max_depth,
					pos,
					args.debug_mode,
				)
			}));
		}
		recv
	};

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

	match args.format {
		FileFormat::Png | FileFormat::Ppm => {
			let mut output_writer: Box<dyn ImageWriter> = match args.format {
				FileFormat::Png => Box::new(PngWriter::new(
					output,
					(image_width, image_height),
					args.bit_depth,
					Some(OffsetDateTime::now_utc()),
					Some(PngRenderingIntent::Perceptual),
				)),
				FileFormat::Ppm => Box::new(PpmWriter::new(
					output,
					(image_width, image_height),
					args.bit_depth,
				)),
				_ => unreachable!(),
			};

			output_writer.write_header()?;
			for mut row in image {
				row.iter_mut().for_each(|p| *p = p.tonemap());
				output_writer.write_pixels(&row)?;
			}
			output_writer.end()?;
		},
		FileFormat::Exr => {
			let channels = AnyChannels::sort(smallvec![
				AnyChannel::new(
					"R",
					channel_from_image::<{ channel::RED }>(&image, args.bit_depth)
				),
				AnyChannel::new(
					"G",
					channel_from_image::<{ channel::GREEN }>(&image, args.bit_depth)
				),
				AnyChannel::new(
					"B",
					channel_from_image::<{ channel::BLUE }>(&image, args.bit_depth)
				),
			]);
			let mut image = Image::from_channels((image_width, image_height), channels);
			// // sRGB
			image.attributes.chromaticities = Some(Chromaticities {
				red: Vec2(0.64, 0.33),
				green: Vec2(0.30, 0.60),
				blue: Vec2(0.15, 0.06),
				white: Vec2(0.3127, 0.3290),
			});
			let mut image_data = io::Cursor::new(vec![0u8; 0]);
			image
				.write()
				.to_buffered(&mut image_data)
				.expect("error writing output image");
			io::copy(&mut image_data.into_inner().as_slice(), &mut output)?;
		},
	}

	Ok(())
}
