use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Args {
	pub threads: usize,
	pub width: usize,
	pub samples: usize,
	pub depth: usize,
	pub seed: u64,
	pub output: Option<String>,
}

const HELP: &'static str = concat!(
	"usage: {bin} [-t|--threads n] [-w|--width w] [-s|--samples s]\n",
	"          [-r|--seed r] [-o|--output filename]\n",
	"\n",
	"  -t, --threads n:       number of threads. default: number of logical processors\n",
	"  -w, --width w:         width of image in pixels. default: 600\n",
	"  -s, --samples s:       number of samples per pixel. default: 100\n",
	"  -d, --depth d:         maximum bounces per ray. default: 50\n",
	"  -r, --seed r:          random number seed. default: entropy from the OS\n",
	"  -o, --output filename: file to output PPM image to. default: stdout\n",
);

pub enum Error {
	PicoError(pico_args::Error),
	UnrecognizedArguments(Vec<OsString>),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::PicoError(e) => e.fmt(f)?,
			Self::UnrecognizedArguments(v) => write!(f, "unrecognized argument(s): {:?}", v)?,
		}
		Ok(())
	}
}

impl From<pico_args::Error> for Error {
	fn from(value: pico_args::Error) -> Self {
		Self::PicoError(value)
	}
}

pub fn show_help() {
	eprint!(
		"{}",
		HELP.replace(
			"{bin}",
			&std::env::args_os().nth(0).unwrap().into_string().unwrap()
		)
	);
}

pub fn parse() -> Result<Args, Error> {
	let mut pargs = pico_args::Arguments::from_env();
	if pargs.contains(["-h", "--help"]) {
		show_help();
		std::process::exit(0);
	}

	let args = Args {
		threads: pargs.opt_value_from_str(["-t", "--threads"])?.unwrap_or(
			std::thread::available_parallelism()
				.unwrap_or(1.try_into().unwrap())
				.get(),
		),
		width: pargs.opt_value_from_str(["-w", "--width"])?.unwrap_or(600),
		samples: pargs
			.opt_value_from_str(["-s", "--samples"])?
			.unwrap_or(100),
		depth: pargs.opt_value_from_str(["-d", "--depth"])?.unwrap_or(50),
		seed: pargs
			.opt_value_from_str(["-r", "--seed"])?
			.unwrap_or_else(|| {
				let mut buf = [0u8; 8];
				getrandom::getrandom(&mut buf).unwrap();
				let seed = u64::from_le_bytes(buf);
				// print out the seed so that users can keep using a seed they like
				eprintln!("using seed: {}", seed);
				seed
			}),
		output: pargs.opt_value_from_str(["-o", "--output"])?,
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

	Ok(args)
}
