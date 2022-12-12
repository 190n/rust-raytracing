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
	pub seed: u64,
	pub output: Option<String>,
	pub scene: WhichScene,
	pub verbose: bool,
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
}

impl FromStr for WhichScene {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"weekend" => Ok(Self::Weekend),
			"gay" => Ok(Self::Gay),
			"tuesday" => Ok(Self::Tuesday),
			"perlin" => Ok(Self::Perlin),
			"earth" => Ok(Self::Earth),
			"cornell" => Ok(Self::Cornell),
			"bisexual" => Ok(Self::Bisexual),
			_ => Err(format!("unknown scene: {}", s)),
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
			"  -r, --seed r:          random number seed. default: entropy from the OS\n",
			"  -o, --output filename: file to output PPM image to. default: stdout\n",
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

	let args = Args {
		threads: pargs
			.opt_value_from_str(["-t", "--threads"])?
			.unwrap_or(system_threads()),
		width: pargs.opt_value_from_str(["-w", "--width"])?.unwrap_or(600),
		samples: pargs
			.opt_value_from_str(["-s", "--samples"])?
			.unwrap_or(100),
		depth: pargs.opt_value_from_str(["-d", "--depth"])?.unwrap_or(50),
		seed: pargs
			.opt_value_from_str(["-r", "--seed"])?
			.map(|seed| Ok::<u64, getrandom::Error>(seed))
			.unwrap_or_else(|| {
				let mut buf = [0u8; 8];
				getrandom(&mut buf)?;
				let seed = u64::from_le_bytes(buf);
				// we will print out the seed so that users can keep using a seed they like
				did_get_seed_from_os = true;
				Ok(seed)
			})?,
		output: pargs.opt_value_from_str(["-o", "--output"])?,
		verbose: pargs.contains(["-v", "--verbose"]),
		scene: pargs
			.opt_value_from_str(["-S", "--scene"])?
			.unwrap_or(WhichScene::Weekend),
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

	let rest = pargs.finish();
	if !rest.is_empty() {
		return Err(Error::UnrecognizedArguments(rest));
	}

	if did_get_seed_from_os {
		eprintln!("using seed: {}", args.seed);
	}

	Ok(args)
}
