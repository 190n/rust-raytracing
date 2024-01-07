use std::f64::consts::PI;
use std::fmt::{self, Debug, Formatter};
use std::path::Path;
use std::sync::Arc;

use image::{DynamicImage, ImageResult};
use once_cell::sync::OnceCell;
use rand::Rng;

use super::Perlin;
use crate::common::{Color, Point3};

pub trait Texture: Debug + Sync + Send {
	fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

pub trait Mappable: Texture {
	type Mapped: Texture;
	fn map(&self, f: &(dyn Fn(Color) -> Color + Send + Sync)) -> Self::Mapped;
}

#[derive(Debug, Clone, Copy)]
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

impl Mappable for SolidColor {
	type Mapped = SolidColor;

	fn map(&self, f: &(dyn Fn(Color) -> Color + Send + Sync)) -> SolidColor {
		SolidColor {
			color_value: f(self.color_value),
		}
	}
}

#[derive(Debug, Clone)]
pub struct CheckerTexture<Odd: Texture, Even: Texture> {
	odd: Odd,
	even: Even,
}

impl<Odd: Texture, Even: Texture> CheckerTexture<Odd, Even> {
	pub fn new(odd: Odd, even: Even) -> Self {
		CheckerTexture { odd, even }
	}
}

impl CheckerTexture<SolidColor, SolidColor> {
	pub fn with_colors(odd: Color, even: Color) -> Self {
		CheckerTexture::new(SolidColor::new(odd), SolidColor::new(even))
	}
}

impl<Odd: Texture, Even: Texture> Texture for CheckerTexture<Odd, Even> {
	fn value(&self, u: f64, v: f64, p: Point3) -> Color {
		let sines = f64::sin(10.0 * p.x()) * f64::sin(10.0 * p.y()) * f64::sin(10.0 * p.z());
		if sines < 0.0 {
			self.odd.value(u, v, p)
		} else {
			self.even.value(u, v, p)
		}
	}
}

impl<Odd: Mappable, Even: Mappable> Mappable for CheckerTexture<Odd, Even> {
	type Mapped = CheckerTexture<Odd::Mapped, Even::Mapped>;

	fn map(&self, f: &(dyn Fn(Color) -> Color + Send + Sync)) -> Self::Mapped {
		CheckerTexture {
			odd: self.odd.map(f),
			even: self.even.map(f),
		}
	}
}

#[derive(Debug, Clone)]
pub struct StripeTexture<T: Texture> {
	stripes: Vec<T>,
	sphere_adjust: bool,
}

macro_rules! flag_cell {
	($colors: expr, $sphere_adjust: expr) => {{
		static FLAG: OnceCell<Arc<StripeTexture<SolidColor>>> = OnceCell::new();
		FLAG.get_or_init(|| Arc::new(StripeTexture::with_colors($colors, $sphere_adjust)))
			.clone()
	}};
}

impl StripeTexture<SolidColor> {
	/// stripes:       which textures to use
	/// sphere_adjust: if true, adjust stripe widths so that each stripe has equal surface area on a
	///                sphere, instead of equal height
	pub fn with_colors(colors: &[Color], sphere_adjust: bool) -> StripeTexture<SolidColor> {
		StripeTexture {
			stripes: Vec::from_iter(colors.iter().map(|&c| SolidColor::new(c))),
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

	fn bi_colors() -> [Color; 5] {
		let pink = Color::from_srgb_hex(0xd60270);
		let purple = Color::from_srgb_hex(0x9b4f96);
		let blue = Color::from_srgb_hex(0x0038a8);
		[pink, pink, purple, blue, blue]
	}
}

impl<T: Texture> StripeTexture<T> {
	pub fn trans_sphere() -> Arc<StripeTexture<SolidColor>> {
		flag_cell!(&StripeTexture::trans_colors(), true)
	}

	pub fn rainbow_sphere() -> Arc<StripeTexture<SolidColor>> {
		flag_cell!(&StripeTexture::rainbow_colors(), true)
	}

	pub fn enby_sphere() -> Arc<StripeTexture<SolidColor>> {
		flag_cell!(&StripeTexture::enby_colors(), true)
	}

	pub fn bi() -> Arc<StripeTexture<SolidColor>> {
		flag_cell!(&StripeTexture::bi_colors(), false)
	}

	pub fn bi_sphere() -> Arc<StripeTexture<SolidColor>> {
		flag_cell!(&StripeTexture::bi_colors(), true)
	}
}

impl<T: Texture> Texture for StripeTexture<T> {
	fn value(&self, u: f64, mut v: f64, p: Point3) -> Color {
		v = 1.0 - v;
		if self.sphere_adjust {
			v = (1.0 - f64::cos(PI * v)) / 2.0;
		}

		let index = ((v * self.stripes.len() as f64) as usize).clamp(0, self.stripes.len() - 1);
		self.stripes[index].value(u, v, p)
	}
}

impl<T: Mappable> Mappable for StripeTexture<T> {
	type Mapped = StripeTexture<T::Mapped>;

	fn map(&self, f: &(dyn Fn(Color) -> Color + Send + Sync)) -> Self::Mapped {
		StripeTexture {
			stripes: Vec::from_iter(self.stripes.iter().map(|s| s.map(f))),
			sphere_adjust: self.sphere_adjust,
		}
	}
}

#[derive(Debug, Clone)]
pub struct NoiseTexture<Low: Texture, High: Texture> {
	noise: Perlin,
	low: Low,
	high: High,
	scale: f64,
	depth: usize,
}

impl<Low: Texture, High: Texture> NoiseTexture<Low, High> {
	pub fn new<R: Rng + ?Sized>(
		rng: &mut R,
		low: Low,
		high: High,
		scale: f64,
		depth: usize,
	) -> Self {
		NoiseTexture {
			noise: Perlin::new(rng),
			low,
			high,
			scale,
			depth,
		}
	}
}

impl<Low: Texture, High: Texture> Texture for NoiseTexture<Low, High> {
	fn value(&self, u: f64, v: f64, p: Point3) -> Color {
		let low = self.low.value(u, v, p);
		let high = self.high.value(u, v, p);
		let value = 0.5
			* (1.0 + f64::sin(self.scale * p.z() + 10.0 * self.noise.turbulence(p, self.depth)));
		low + (high - low) * value
	}
}

impl<Low: Mappable, High: Mappable> Mappable for NoiseTexture<Low, High> {
	type Mapped = NoiseTexture<Low::Mapped, High::Mapped>;

	fn map(&self, f: &(dyn Fn(Color) -> Color + Send + Sync)) -> Self::Mapped {
		NoiseTexture {
			noise: self.noise.clone(),
			low: self.low.map(f),
			high: self.high.map(f),
			scale: self.scale,
			depth: self.depth,
		}
	}
}

#[derive(Debug, Clone)]
pub struct ImageTexture {
	image: DynamicImage,
}

impl ImageTexture {
	pub fn new(filename: impl AsRef<Path>) -> ImageResult<ImageTexture> {
		Ok(ImageTexture {
			image: image::open(filename)?,
		})
	}
}

impl Texture for ImageTexture {
	fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
		let u = u.clamp(0.0, 1.0);
		let v = 1.0 - v.clamp(0.0, 1.0);

		let i = ((u * self.image.width() as f64) as u32).clamp(0, self.image.width() - 1);
		let j = ((v * self.image.height() as f64) as u32).clamp(0, self.image.height() - 1);

		match &self.image {
			DynamicImage::ImageRgb8(im) => {
				let pix = im.get_pixel(i, j);
				Color::from_srgb(pix.0[0], pix.0[1], pix.0[2])
			},
			_ => panic!("bad image type: {:?}", self.image.color()),
		}
	}
}

#[derive(Clone)]
pub struct FunctionTexture<F: Fn(f64, f64, Point3) -> Color + Send + Sync>(pub F);

impl<F: Fn(f64, f64, Point3) -> Color + Send + Sync> Debug for FunctionTexture<F> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "FunctionTexture")
	}
}

impl<F: Fn(f64, f64, Point3) -> Color + Send + Sync> Texture for FunctionTexture<F> {
	fn value(&self, u: f64, v: f64, p: Point3) -> Color {
		(self.0)(u, v, p)
	}
}
