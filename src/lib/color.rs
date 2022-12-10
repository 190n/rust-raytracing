use std::io::{self, Write};

use super::Vec3;

pub type Color = Vec3;

fn srgb_to_linear(value: f64) -> f64 {
	if value <= 0.04045 {
		value / 12.92
	} else {
		((value + 0.055) / 1.055).powf(2.4)
	}
}

fn linear_to_srgb(value: f64) -> f64 {
	if value <= 0.0031308 {
		12.92 * value
	} else {
		1.055 * value.powf(1.0 / 2.4) - 0.055
	}
}

impl Color {
	pub fn from_srgb(r: u8, g: u8, b: u8) -> Color {
		Color::new(
			srgb_to_linear(r as f64 / 255.0),
			srgb_to_linear(g as f64 / 255.0),
			srgb_to_linear(b as f64 / 255.0),
		)
	}

	pub fn from_srgb_hex(code: u32) -> Color {
		Color::from_srgb(
			((code & 0xff0000) >> 16) as u8,
			((code & 0x00ff00) >> 8) as u8,
			((code & 0x0000ff) >> 0) as u8,
		)
	}
}

pub fn write_color(w: &mut impl Write, color: Color, samples_per_pixel: usize) -> io::Result<()> {
	let mut r = color.x();
	let mut g = color.y();
	let mut b = color.z();

	let scale = 1.0 / samples_per_pixel as f64;
	r = linear_to_srgb(r * scale);
	g = linear_to_srgb(g * scale);
	b = linear_to_srgb(b * scale);

	let r = (256.0 * r.clamp(0.0, 0.999)) as u8;
	let g = (256.0 * g.clamp(0.0, 0.999)) as u8;
	let b = (256.0 * b.clamp(0.0, 0.999)) as u8;

	w.write(&[r, g, b])?;
	Ok(())
}
