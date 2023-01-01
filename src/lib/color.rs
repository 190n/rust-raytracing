use super::Vec3;

pub type Color = Vec3;

// Color (f64)
// `-> oetf -> f64
// `-> tonemap -> u16
// `-> dither -> u16

#[derive(Clone, Copy)]
pub struct OutputColor(pub u16, pub u16, pub u16);

pub struct Dither {
	/// peak value to output
	scale: f64,
	/// quantization errors spread to nearby pixels
	errors: [Vec<Color>; 2],
	x: usize,
	width: usize,
}

impl Dither {
	/// Create a new ditherer to reduce colors from 16 to the specified number of bits, which must
	/// be within [1, 16], and with the width of the image for error diffusion
	pub fn new(bits: u8, width: usize) -> Dither {
		if bits > 16 || bits < 1 {
			panic!("number of bits for dither must be between 1 and 16");
		}
		Dither {
			scale: ((1 << bits) - 1) as f64,
			errors: [vec![Color::zero(); width], vec![Color::zero(); width]],
			x: 0,
			width,
		}
	}

	fn spread_error(&mut self, error: Color, offset_x: isize, offset_y: usize, factor: f64) {
		let x = (self.x as isize) + offset_x;
		if x < 0 || x >= self.width as isize {
			return;
		}
		let x = x as usize;
		self.errors[offset_y][x] += error * factor;
	}

	fn round(&self, input: Color) -> Color {
		Color::new(
			(input.x() * self.scale).round() / self.scale,
			(input.y() * self.scale).round() / self.scale,
			(input.z() * self.scale).round() / self.scale,
		)
	}

	fn advance(&mut self) {
		if self.x == self.width - 1 {
			self.errors.swap(0, 1);
			self.errors[1].fill(Color::zero());
		}
		self.x = (self.x + 1) % self.width;
	}

	pub fn dither(&mut self, input: Color) -> OutputColor {
		let old_pixel = input + self.errors[0][self.x];
		let new_pixel = self.round(old_pixel);
		let quant_error = old_pixel - new_pixel;

		self.spread_error(quant_error, 1, 0, 7.0 / 16.0);
		self.spread_error(quant_error, -1, 1, 3.0 / 16.0);
		self.spread_error(quant_error, 0, 1, 5.0 / 16.0);
		self.spread_error(quant_error, 1, 1, 1.0 / 16.0);

		self.advance();
		OutputColor(
			(new_pixel.x() * self.scale) as u16,
			(new_pixel.y() * self.scale) as u16,
			(new_pixel.z() * self.scale) as u16,
		)
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

	fn clamp(&self) -> Color {
		Color::new(
			self.x().clamp(0.0, 1.0),
			self.y().clamp(0.0, 1.0),
			self.z().clamp(0.0, 1.0),
		)
	}

	pub fn tonemap(&self) -> Color {
		self.oetf().clamp()
	}
}
