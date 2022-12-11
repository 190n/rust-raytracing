use once_cell::sync::OnceCell;
use rand::Rng;
use std::f64::consts::PI;
use std::fmt::Debug;
use std::sync::Arc;

use super::Perlin;
use crate::lib::{Color, Point3};

pub trait Texture: Debug + Sync + Send {
	fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

#[derive(Debug)]
pub struct SolidColor {
	color_value: Color,
}

impl SolidColor {
	pub fn new(color_value: Color) -> SolidColor {
		SolidColor { color_value }
	}
}

impl Texture for SolidColor {
	fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
		self.color_value
	}
}

#[derive(Debug)]
pub struct CheckerTexture {
	odd: Arc<dyn Texture>,
	even: Arc<dyn Texture>,
}

impl CheckerTexture {
	pub fn new(odd: Arc<dyn Texture>, even: Arc<dyn Texture>) -> CheckerTexture {
		CheckerTexture { odd, even }
	}

	pub fn with_colors(odd: Color, even: Color) -> CheckerTexture {
		CheckerTexture::new(
			Arc::new(SolidColor::new(odd)),
			Arc::new(SolidColor::new(even)),
		)
	}
}

impl Texture for CheckerTexture {
	fn value(&self, u: f64, v: f64, p: Point3) -> Color {
		let sines = f64::sin(10.0 * p.x()) * f64::sin(10.0 * p.y()) * f64::sin(10.0 * p.z());
		if sines < 0.0 {
			self.odd.value(u, v, p)
		} else {
			self.even.value(u, v, p)
		}
	}
}

#[derive(Debug)]
pub struct StripeTexture {
	stripes: Vec<Arc<dyn Texture>>,
	sphere_adjust: bool,
}

macro_rules! flag_cell {
	($colors: expr, $sphere_adjust: expr) => {{
		static FLAG: OnceCell<Arc<dyn Texture>> = OnceCell::new();
		FLAG.get_or_init(|| Arc::new(StripeTexture::with_colors($colors, $sphere_adjust)))
			.clone()
	}};
}

impl StripeTexture {
	/// stripes:       which textures to use
	/// sphere_adjust: if true, adjust stripe widths so that each stripe has equal surface area on a
	///                sphere, instead of equal height
	pub fn new(stripes: &[Arc<dyn Texture>], sphere_adjust: bool) -> StripeTexture {
		StripeTexture {
			stripes: Vec::from(stripes),
			sphere_adjust,
		}
	}

	fn trans_colors() -> [Color; 5] {
		let blue = Color::from_srgb_hex(0x5bcefa);
		let pink = Color::from_srgb_hex(0xf5a9b8);
		let white = Color::from_srgb_hex(0xffffff);
		[blue, pink, white, pink, blue]
	}

	fn rainbow_colors() -> [Color; 6] {
		let red = Color::from_srgb_hex(0xe40303);
		let orange = Color::from_srgb_hex(0xff8c00);
		let yellow = Color::from_srgb_hex(0xffed00);
		let green = Color::from_srgb_hex(0x008026);
		let blue = Color::from_srgb_hex(0x004dff);
		let purple = Color::from_srgb_hex(0x750787);
		[red, orange, yellow, green, blue, purple]
	}

	fn enby_colors() -> [Color; 4] {
		let yellow = Color::from_srgb_hex(0xfef333);
		let white = Color::from_srgb_hex(0xffffff);
		let purple = Color::from_srgb_hex(0x9a58cf);
		let gray = Color::from_srgb_hex(0x2d2d2d);
		[yellow, white, purple, gray]
	}

	/// stripes:       which textures to use
	/// sphere_adjust: if true, adjust stripe widths so that each stripe has equal surface area on a
	///                sphere, instead of equal height
	pub fn with_colors(colors: &[Color], sphere_adjust: bool) -> StripeTexture {
		StripeTexture {
			stripes: Vec::from_iter(
				colors
					.iter()
					.map(|&c| Arc::new(SolidColor::new(c)) as Arc<dyn Texture>),
			),
			sphere_adjust,
		}
	}

	pub fn trans() -> Arc<dyn Texture> {
		flag_cell!(&StripeTexture::trans_colors(), false)
	}

	pub fn trans_sphere() -> Arc<dyn Texture> {
		flag_cell!(&StripeTexture::trans_colors(), true)
	}

	pub fn rainbow() -> Arc<dyn Texture> {
		flag_cell!(&StripeTexture::rainbow_colors(), false)
	}

	pub fn rainbow_sphere() -> Arc<dyn Texture> {
		flag_cell!(&StripeTexture::rainbow_colors(), true)
	}

	pub fn enby() -> Arc<dyn Texture> {
		flag_cell!(&StripeTexture::enby_colors(), false)
	}

	pub fn enby_sphere() -> Arc<dyn Texture> {
		flag_cell!(&StripeTexture::enby_colors(), true)
	}
}

impl Texture for StripeTexture {
	fn value(&self, u: f64, mut v: f64, p: Point3) -> Color {
		if self.sphere_adjust {
			v = (1.0 - f64::cos(PI * (1.0 - v))) / 2.0;
		}

		let index = ((v * self.stripes.len() as f64) as usize).clamp(0, self.stripes.len() - 1);
		self.stripes[index].value(u, v, p)
	}
}

#[derive(Debug)]
pub struct NoiseTexture {
	noise: Perlin,
	low: Arc<dyn Texture>,
	high: Arc<dyn Texture>,
	scale: f64,
}

impl NoiseTexture {
	pub fn new<R: Rng + ?Sized>(
		rng: &mut R,
		low: Arc<dyn Texture>,
		high: Arc<dyn Texture>,
		scale: f64,
	) -> NoiseTexture {
		NoiseTexture {
			noise: Perlin::new(rng),
			low,
			high,
			scale,
		}
	}
}

impl Texture for NoiseTexture {
	fn value(&self, u: f64, v: f64, p: Point3) -> Color {
		let low = self.low.value(u, v, p);
		let high = self.high.value(u, v, p);
		low + (high - low) * self.noise.noise(self.scale * p)
	}
}
