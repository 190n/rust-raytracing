pub mod png;
mod ppm;

pub use png::PngWriter;
pub use ppm::PpmWriter;

use std::io;

use crate::lib::Color;

pub trait ImageWriter {
	fn write_header(&mut self) -> io::Result<()>;
	fn write_pixels(&mut self, pixels: &[Color]) -> io::Result<()>;
	fn end(&mut self) -> io::Result<()>;
}
