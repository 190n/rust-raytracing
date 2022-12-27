pub mod png;
mod ppm;

pub use png::PngWriter;
pub use ppm::PpmWriter;

use std::io;

use crate::lib::color::OutputColor;

pub trait ImageWriter {
	fn write_header(&mut self) -> io::Result<()>;
	fn write_pixels(&mut self, pixels: &[OutputColor]) -> io::Result<()>;
	fn end(self) -> io::Result<()>;
}
