use crate::vec::Color;
use std::io::{self, Write};

pub fn write_color(w: &mut impl Write, color: Color, samples_per_pixel: usize) -> io::Result<()> {
	let mut r = color.x();
	let mut g = color.y();
	let mut b = color.z();

	let scale = 1.0 / samples_per_pixel as f64;
	r = (r * scale).sqrt();
	g = (g * scale).sqrt();
	b = (b * scale).sqrt();

	let r = (256.0 * r.clamp(0.0, 0.999)) as u8;
	let g = (256.0 * g.clamp(0.0, 0.999)) as u8;
	let b = (256.0 * b.clamp(0.0, 0.999)) as u8;

	w.write(&[r, g, b])?;
	Ok(())
}
