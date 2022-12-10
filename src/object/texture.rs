use once_cell::sync::OnceCell;
use std::fmt::Debug;
use std::sync::Arc;

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
}

impl StripeTexture {
	pub fn new(stripes: &[Arc<dyn Texture>]) -> StripeTexture {
		StripeTexture {
			stripes: Vec::from(stripes),
		}
	}

	pub fn with_colors(colors: &[Color]) -> StripeTexture {
		StripeTexture {
			stripes: Vec::from_iter(
				colors
					.iter()
					.map(|&c| Arc::new(SolidColor::new(c)) as Arc<dyn Texture>),
			),
		}
	}

	pub fn trans() -> Arc<dyn Texture> {
		static TRANS_FLAG: OnceCell<Arc<dyn Texture>> = OnceCell::new();
		TRANS_FLAG
			.get_or_init(|| {
				let blue = Color::from_srgb_hex(0x5bcefa);
				let pink = Color::from_srgb_hex(0xf5a9b8);
				let white = Color::from_srgb_hex(0xffffff);
				Arc::new(StripeTexture::with_colors(&[blue, pink, white, pink, blue]))
			})
			.clone()
	}
}

impl Texture for StripeTexture {
	fn value(&self, u: f64, v: f64, p: Point3) -> Color {
		let index = ((v * self.stripes.len() as f64) as usize).clamp(0, self.stripes.len() - 1);
		self.stripes[index].value(u, v, p)
	}
}
