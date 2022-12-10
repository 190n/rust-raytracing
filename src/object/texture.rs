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
