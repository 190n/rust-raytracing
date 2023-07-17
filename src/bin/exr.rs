use exr::prelude::*;
use std::io::Cursor;

fn main() {
	let image = read_first_rgba_layer_from_file(
		"final.exr",
		|resolution, _| vec![vec![(0.0, 0.0, 0.0); resolution.width()]; resolution.height()],
		|image: &mut Vec<Vec<(f32, f32, f32)>>,
		 pos: Vec2<usize>,
		 (r, g, b, _a): (f32, f32, f32, f32)| { image[pos.y()][pos.x()] = (r, g, b) },
	)
	.unwrap();
	let mut output = Cursor::new(vec![0u8; 0]);
	let channels = SpecificChannels::rgb(|pos: Vec2<usize>| {
		let p = image.layer_data.channel_data.pixels[pos.1][pos.0];
		(f16::from_f32(p.0), f16::from_f32(p.1), f16::from_f32(p.2))
	});
	let layer = Layer::new(
		(1080, 1080),
		LayerAttributes::default(),
		Encoding::SMALL_LOSSLESS,
		channels,
	);
	let mut output_image = Image::from_layer(layer);
	output_image.attributes.chromaticities = Some(attribute::Chromaticities {
		red: Vec2(0.64, 0.33),
		green: Vec2(0.30, 0.60),
		blue: Vec2(0.15, 0.06),
		white: Vec2(0.3127, 0.3290),
	});
	output_image.write().to_buffered(&mut output).unwrap();
	std::io::copy(&mut output.into_inner().as_slice(), &mut std::io::stdout()).unwrap();
}
