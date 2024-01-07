use std::io::{self, Write};

use once_cell::unsync::Lazy;
use time::{OffsetDateTime, UtcOffset};

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum PngRenderingIntent {
	Perceptual = 0,
	RelativeColorimetric = 1,
	Saturation = 2,
	AbsoluteColorimetric = 3,
}

#[allow(dead_code)]
pub enum TextData {
	Uncompressed(String),
	Compressed(Vec<u8>),
}

#[allow(dead_code)]
pub enum PngChunk<'a> {
	Ihdr {
		width: u32,
		height: u32,
		bit_depth: u8,
	},
	Idat(&'a [u8]),
	Iend,
	Sbit(u8),
	Gama(f64),
	Srgb(PngRenderingIntent),
	Itxt {
		keyword: String,
		language: Option<String>,
		translated_keyword: Option<String>,
		text: TextData,
	},
	Time(OffsetDateTime),
}

struct Crc32<W: Write>(u32, W);

impl<W: Write> Crc32<W> {
	pub fn new(dest: W) -> Self {
		Crc32(0xffffffff, dest)
	}

	fn update(&mut self, buf: &[u8]) {
		let table: Lazy<[u32; 256]> = Lazy::new(|| {
			let mut table = [0u32; 256];
			for i in 0..256 {
				let mut c: u32 = i;
				for _ in 0..8 {
					if c & 1 != 0 {
						c = 0xedb88320 ^ (c >> 1);
					} else {
						c = c >> 1;
					}
				}
				table[i as usize] = c;
			}
			table
		});

		for &b in buf {
			self.0 = table[((self.0 ^ b as u32) & 0xff) as usize] ^ (self.0 >> 8);
		}
	}

	pub fn get(&self) -> u32 {
		!self.0
	}
}

impl<W: Write> Write for Crc32<W> {
	fn flush(&mut self) -> io::Result<()> {
		self.1.flush()
	}

	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let res = self.1.write(buf);
		if let Ok(n) = res {
			self.update(&buf[..n]);
		}
		res
	}
}

impl<'a> PngChunk<'a> {
	fn tag(&self) -> &'static [u8; 4] {
		match self {
			PngChunk::Ihdr {
				width: _,
				height: _,
				bit_depth: _,
			} => b"IHDR",
			PngChunk::Idat(_) => b"IDAT",
			PngChunk::Iend => b"IEND",
			PngChunk::Sbit(_) => b"sBIT",
			PngChunk::Gama(_) => b"gAMA",
			PngChunk::Srgb(_) => b"sRGB",
			PngChunk::Itxt {
				keyword: _,
				language: _,
				translated_keyword: _,
				text: _,
			} => b"iTXt",
			PngChunk::Time(_) => b"tIME",
		}
	}

	fn len(&self) -> usize {
		match self {
			PngChunk::Ihdr {
				width: _,
				height: _,
				bit_depth: _,
			} => 13,
			PngChunk::Idat(data) => data.len(),
			PngChunk::Iend => 0,
			PngChunk::Sbit(_) => 3,
			PngChunk::Gama(_) => 4,
			PngChunk::Srgb(_) => 1,
			PngChunk::Itxt {
				keyword,
				language,
				translated_keyword,
				text,
			} => {
				keyword.len()
					// null, compression flag, compression method
					+ 1 + 1 + 1 + if let Some(s) = language { s.len() } else { 0 }
					// null
					+ 1 + if let Some(s) = translated_keyword {
						s.len()
					} else {
						0
					}
					// null
					+ 1 + match text {
						TextData::Uncompressed(s) => s.len(),
						TextData::Compressed(v) => v.len(),
					}
			},
			PngChunk::Time(_) => 7,
		}
	}

	pub fn write_to(&self, w: &mut impl Write) -> io::Result<()> {
		w.write_all(&(self.len() as u32).to_be_bytes())?;
		let mut crc = Crc32::new(w);
		crc.write_all(self.tag())?;

		match self {
			&PngChunk::Ihdr {
				width,
				height,
				bit_depth,
			} => {
				crc.write_all(&width.to_be_bytes())?;
				crc.write_all(&height.to_be_bytes())?;
				crc.write_all(&[
					bit_depth, 2, // color type 2 = truecolor
					0, // compression method 0 = deflate
					0, // filter method 0 = adaptive with 5 types
					0, // interlace method 0 = not interlaced
				])?;
			},
			PngChunk::Idat(data) => {
				crc.write_all(data)?;
			},
			PngChunk::Iend => {},
			&PngChunk::Sbit(bits) => {
				// write it three times for red, green, and blue
				crc.write_all(&[bits, bits, bits])?;
			},
			&PngChunk::Gama(gamma) => {
				let integer_gamma = (gamma * 100_000.0) as u32;
				crc.write_all(&integer_gamma.to_be_bytes())?;
			},
			&PngChunk::Srgb(intent) => {
				crc.write_all(&[intent as u8])?;
			},
			PngChunk::Itxt {
				keyword,
				language,
				translated_keyword,
				text,
			} => {
				crc.write_all(keyword.as_bytes())?;
				crc.write_all(&[
					0, // null
					match text {
						// compression flag
						TextData::Compressed(_) => 1,
						TextData::Uncompressed(_) => 0,
					},
					0, //compression method
				])?;
				if let Some(s) = language {
					crc.write_all(s.as_bytes())?;
				}
				crc.write_all(&[0])?; // null
				if let Some(s) = translated_keyword {
					crc.write_all(s.as_bytes())?;
				}
				crc.write_all(&[0])?; // null
				crc.write_all(match text {
					TextData::Compressed(v) => v,
					TextData::Uncompressed(s) => s.as_bytes(),
				})?;
			},
			PngChunk::Time(time) => {
				let utc_time = time.to_offset(UtcOffset::UTC);
				crc.write_all(&(utc_time.year() as u16).to_be_bytes())?;
				crc.write_all(&[
					utc_time.month() as u8,
					utc_time.day(),
					utc_time.hour(),
					utc_time.minute(),
					utc_time.second(),
				])?;
			},
		}

		// we write to the CRC here since it's easier than accessing the original stream that is now
		// wrapped in the CRC
		// this does modify the hash state, but we access the hash before doing this write
		crc.write_all(&crc.get().to_be_bytes())
	}
}
