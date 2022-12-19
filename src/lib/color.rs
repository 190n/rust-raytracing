use super::Vec3;

pub type Color = Vec3;

// Color (f64)
// `-> oetf -> f64
// `-> tonemap -> u16
// `-> dither -> u16

#[derive(Clone, Copy)]
pub struct OutputColor(pub u16, pub u16, pub u16);

pub struct Dither {
	bits: u8,
}

impl Dither {
	/// Create a new ditherer to reduce colors from 16 to the specified number of bits, which must
	/// be within [1, 16].
	pub fn new(bits: u8) -> Dither {
		if bits > 16 || bits < 1 {
			panic!("number of bits for dither must be between 1 and 16");
		}
		Dither { bits }
	}

	pub fn dither(&mut self, input: OutputColor) -> OutputColor {
		let shift = 16 - self.bits;
		OutputColor(input.0 >> shift, input.1 >> shift, input.2 >> shift)
	}
}

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

fn clamp(sample: f64) -> u16 {
	(65536.0 * sample.clamp(0.0, 65535.0 / 65536.0)) as u16
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

	fn oetf(&self) -> Color {
		Color::new(
			linear_to_srgb(self.x()),
			linear_to_srgb(self.y()),
			linear_to_srgb(self.z()),
		)
	}

	fn clamp(&self) -> OutputColor {
		OutputColor(clamp(self.x()), clamp(self.y()), clamp(self.z()))
	}

	pub fn tonemap(&self) -> OutputColor {
		self.oetf().clamp()
	}
}
