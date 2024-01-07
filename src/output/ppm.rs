use std::io::{self, BufWriter, Write};

use super::ImageWriter;
use crate::common::color::{Color, Dither};

pub struct PpmWriter<W: Write> {
	dest: BufWriter<W>,
	width: usize,
	height: usize,
	dither: Dither,
	max: usize,
}

impl<W: Write> PpmWriter<W> {
	pub fn new(dest: W, (width, height): (usize, usize), bits: u8) -> Self {
		if bits > 8 {
			panic!("PPM only supports up to 8 bits per channel");
		}
		Self {
			dest: BufWriter::new(dest),
			width,
			height,
			dither: Dither::new(bits, width),
			max: (1 << bits) - 1,
		}
	}
}

impl<W: Write> ImageWriter for PpmWriter<W> {
	fn write_header(&mut self) -> io::Result<()> {
		write!(
			self.dest,
			"P6\n{} {}\n{}\n",
			self.width, self.height, self.max
		)
	}

	fn write_pixels(&mut self, pixels: &[Color]) -> io::Result<()> {
		for p in pixels.iter().map(|&p| self.dither.dither(p)) {
			self.dest.write_all(&[p.0 as u8, p.1 as u8, p.2 as u8])?;
		}
		Ok(())
	}

	fn end(&mut self) -> io::Result<()> {
		self.dest.flush()
	}
}
