use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use getrandom::getrandom;

#[derive(Debug)]
pub struct Args {
	pub threads: usize,
	pub width: usize,
	pub samples: usize,
	pub depth: usize,
	pub world_seed: u64,
	pub sample_seed: u64,
	pub output: Option<String>,
	pub scene: WhichScene,
	pub verbose: bool,
	pub format: FileFormat,
	pub bit_depth: u8,
}

pub struct ParseEnumError(pub &'static str);

impl Display for ParseEnumError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "unknown {}", self.0)
	}
}

#[derive(Debug)]
pub enum WhichScene {
	Weekend,
	Gay,
	Tuesday,
	Perlin,
	Earth,
	Cornell,
	Bisexual,
	Week,
}

impl FromStr for WhichScene {
	type Err = ParseEnumError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"weekend" => Ok(Self::Weekend),
			"gay" => Ok(Self::Gay),
			"tuesday" => Ok(Self::Tuesday),
			"perlin" => Ok(Self::Perlin),
			"earth" => Ok(Self::Earth),
			"cornell" => Ok(Self::Cornell),
			"bisexual" => Ok(Self::Bisexual),
			"week" => Ok(Self::Week),
			_ => Err(ParseEnumError("scene")),
		}
	}
}

#[derive(Debug)]
pub enum FileFormat {
	Png,
	Ppm,
}

impl FileFormat {
	pub fn from_extension(filename: &str) -> Result<FileFormat, ParseEnumError> {
		if filename.ends_with(".png") {
			Ok(FileFormat::Png)
		} else if filename.ends_with(".ppm") {
			Ok(FileFormat::Ppm)
		} else {
			Err(ParseEnumError("format"))
		}
	}
}

impl FromStr for FileFormat {
	type Err = ParseEnumError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"png" => Ok(Self::Png),
			"ppm" => Ok(Self::Ppm),
			_ => Err(ParseEnumError("format")),
		}
	}
}

pub enum Error {
	PicoError(pico_args::Error),
	UnrecognizedArguments(Vec<OsString>),
	GetrandomError(getrandom::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::PicoError(e) => e.fmt(f)?,
			Self::UnrecognizedArguments(v) => write!(f, "unrecognized argument(s): {:?}", v)?,
			Self::GetrandomError(e) => write!(f, "error generating entropy: {}", e)?,
		}
		Ok(())
	}
}

impl From<pico_args::Error> for Error {
	fn from(value: pico_args::Error) -> Self {
		Self::PicoError(value)
	}
}

impl From<getrandom::Error> for Error {
	fn from(value: getrandom::Error) -> Self {
		Self::GetrandomError(value)
	}
}

fn system_threads() -> usize {
	std::thread::available_parallelism()
		.unwrap_or(1.try_into().unwrap())
		.get()
}

fn entropy_seed() -> Result<u64, getrandom::Error> {
	let mut buf = [0u8; 8];
	getrandom(&mut buf)?;
	Ok(u64::from_le_bytes(buf))
}

pub fn show_help() {
	eprint!(
		concat!(
			"usage: {} [-t|--threads n] [-w|--width w] [-s|--samples s] [-r|--seed r] \n",
			"         [-d|--depth d] [-o|--output filename] [-S|--scene scene]\n",
			"\n",
			"  -t, --threads n:       number of threads. default: number of logical processors ({})\n",
			"  -w, --width w:         width of image in pixels. default: 600\n",
			"  -s, --samples s:       number of samples per pixel. default: 100\n",
			"  -d, --depth d:         maximum bounces per ray. default: 50\n",
			"  -r, --world-seed n:    random number seed for generating the world.\n",
			"                         default: entropy from the OS\n",
			"  -R, --sample-seed n:   random number seed for shooting rays.\n",
			"                         default: entropy from the OS\n",
			"  -o, --output filename: file to output image to. default: stdout\n",
			"  -f, --format png|ppm:  which format to output. default: guess from file extension,\n",
			"                         or PPM for stdout\n",
			"  -b, --bit-depth n:     number of bits per channel in the output image. default: 8.\n",
			"                         range: 1-8 for PPM, 1-16 for PNG.\n",
			"  -v, --verbose:         log performance data to stderr\n",
			"  -S, --scene scene:     which scene to render. options:\n",
			"    weekend:\n",
			"      random spheres; final render from Ray Tracing in One Weekend\n",
			"    gay:\n",
			"      the random spheres scene, but with pride flag textures on the small spheres\n",
			"    tuesday:\n",
			"      the random spheres scene, but upgraded with features from The Next Week:\n",
			"        - moving spheres\n",
			"        - checkered ground texture\n",
			"    perlin:\n",
			"      two spheres with Perlin noise\n",
			"    earth:\n",
			"      a globe with the texture of the Earth\n",
			"    cornell:\n",
			"      the Cornell box\n",
			"    bisexual:\n",
			"      the Cornell box but with bisexual lighting\n",
			"    week:\n",
			"      final scene from Ray Tracing: The Next Week\n",
			"    default: weekend\n",
		),
		std::env::args_os()
			.nth(0)
			.unwrap_or_else(|| "raytracing".into())
			.into_string()
			.unwrap_or_else(|_| "raytracing".into()),
		system_threads()
	);
}

pub fn parse() -> Result<Args, Error> {
	let mut pargs = pico_args::Arguments::from_env();
	if pargs.contains(["-h", "--help"]) {
		show_help();
		std::process::exit(0);
	}

	let mut did_get_seed_from_os = false;
	let mut guess_format = false;

	let mut args = Args {
		threads: pargs
			.opt_value_from_str(["-t", "--threads"])?
			.unwrap_or(system_threads()),
		width: pargs.opt_value_from_str(["-w", "--width"])?.unwrap_or(600),
		samples: pargs
			.opt_value_from_str(["-s", "--samples"])?
			.unwrap_or(100),
		depth: pargs.opt_value_from_str(["-d", "--depth"])?.unwrap_or(50),
		world_seed: pargs
			.opt_value_from_str(["-r", "--world-seed"])?
			.map(|seed| Ok::<u64, getrandom::Error>(seed))
			.unwrap_or_else(|| {
				// we will print out the seed so that users can keep using a seed they like
				did_get_seed_from_os = true;
				entropy_seed()
			})?,
		sample_seed: pargs
			.opt_value_from_str(["-R", "--sample-seed"])?
			.map(|seed| Ok::<u64, getrandom::Error>(seed))
			.unwrap_or_else(|| {
				did_get_seed_from_os = true;
				entropy_seed()
			})?,
		output: pargs.opt_value_from_str(["-o", "--output"])?,
		verbose: pargs.contains(["-v", "--verbose"]),
		scene: pargs
			.opt_value_from_str(["-S", "--scene"])?
			.unwrap_or(WhichScene::Weekend),
		format: pargs
			.opt_value_from_str(["-f", "--format"])?
			.unwrap_or_else(|| {
				guess_format = true;
				FileFormat::Ppm
			}),
		bit_depth: pargs
			.opt_value_from_str(["-b", "--bit-depth"])?
			.unwrap_or(8),
	};

	if args.threads == 0 {
		return Err(Error::PicoError(
			pico_args::Error::Utf8ArgumentParsingFailed {
				value: "0".to_string(),
				cause: "number of threads must be nonzero".to_string(),
			},
		));
	}
	if let Some(ref s) = args.output {
		if s.is_empty() {
			return Err(Error::PicoError(
				pico_args::Error::Utf8ArgumentParsingFailed {
					value: s.to_string(),
					cause: "output filename must not be empty".to_string(),
				},
			));
		}
	}

	if guess_format {
		if let Some(ref s) = args.output {
			if let Ok(format) = FileFormat::from_extension(s) {
				args.format = format;
			} else {
				return Err(Error::PicoError(
					pico_args::Error::Utf8ArgumentParsingFailed {
						value: s.to_string(),
						cause: "failed to determine format from extension".to_string(),
					},
				));
			}
		}
	}

	match args.format {
		FileFormat::Png => {
			if args.bit_depth < 1 || args.bit_depth > 16 {
				return Err(Error::PicoError(
					pico_args::Error::Utf8ArgumentParsingFailed {
						value: args.bit_depth.to_string(),
						cause: "PNG image bit depth must be between 1 and 16".to_string(),
					},
				));
			}
		},
		FileFormat::Ppm => {
			if args.bit_depth < 1 || args.bit_depth > 8 {
				return Err(Error::PicoError(
					pico_args::Error::Utf8ArgumentParsingFailed {
						value: args.bit_depth.to_string(),
						cause: "PPM image bit depth must be between 1 and 8".to_string(),
					},
				));
			}
		},
	}

	let rest = pargs.finish();
	if !rest.is_empty() {
		return Err(Error::UnrecognizedArguments(rest));
	}

	if did_get_seed_from_os {
		eprintln!(
			"using seeds: -r {} -R {}",
			args.world_seed, args.sample_seed
		);
	}

	Ok(args)
}
