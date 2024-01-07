mod chunk;

use std::io::{self, BufWriter, Write};

use flate2::write::ZlibEncoder;
use flate2::Compression;
use time::OffsetDateTime;

use super::ImageWriter;
use crate::common::color::{Color, Dither};
use chunk::PngChunk;

pub use chunk::PngRenderingIntent;

const IDAT_SIZE: usize = 8192;

/// writes slices to the underlying writer in the form of IDAT chunks
struct IdatWriter<W: Write>(pub W);

impl<W: Write> IdatWriter<W> {
	pub fn finish(mut self) -> io::Result<W> {
		self.0.flush()?;
		Ok(self.0)
	}
}

impl<W: Write> Write for IdatWriter<W> {
	fn flush(&mut self) -> io::Result<()> {
		self.0.flush()
	}

	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		PngChunk::Idat(buf).write_to(&mut self.0)?;
		Ok(buf.len())
	}
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterType {
	None = 0,
}

/// writes scanlines to the underlying writer, filtering them and prepending a filter byte before
/// each one
struct FilterWriter<W: Write> {
	dest: W,
	current_filter: FilterType,
	next_filter: FilterType,
	scanline_size: usize,
	scanline_pos: usize,
}

impl<W: Write> FilterWriter<W> {
	pub fn new(dest: W, filter: FilterType, scanline_size: usize) -> Self {
		Self {
			dest,
			current_filter: filter,
			next_filter: filter,
			scanline_size,
			scanline_pos: 0,
		}
	}

	pub fn finish(mut self) -> io::Result<W> {
		self.dest.flush()?;
		Ok(self.dest)
	}
}

impl<W: Write> Write for FilterWriter<W> {
	fn flush(&mut self) -> io::Result<()> {
		self.dest.flush()
	}

	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let amount = usize::min(buf.len(), self.scanline_size - self.scanline_pos);
		if self.scanline_pos == 0 {
			self.current_filter = self.next_filter;
			self.dest.write_all(&[self.current_filter as u8])?;
		}

		self.dest.write_all(&buf[..amount])?;
		self.scanline_pos = (self.scanline_pos + amount) % self.scanline_size;

		Ok(amount)
	}
}

pub struct PngWriter<W: Write> {
	buf: Option<BufWriter<W>>,
	pixel_writer: Option<BufWriter<FilterWriter<ZlibEncoder<IdatWriter<BufWriter<W>>>>>>,
	width: usize,
	height: usize,
	bits: u8,
	time: Option<OffsetDateTime>,
	srgb: Option<PngRenderingIntent>,
	dither: Dither,
}

impl<W: Write> PngWriter<W> {
	pub fn new(
		dest: W,
		(width, height): (usize, usize),
		bits: u8,
		time: Option<OffsetDateTime>,
		srgb: Option<PngRenderingIntent>,
	) -> Self {
		assert!(bits > 0 && bits <= 16);
		Self {
			buf: Some(BufWriter::with_capacity(IDAT_SIZE + 12, dest)),
			pixel_writer: None,
			width,
			height,
			bits,
			time,
			srgb,
			dither: Dither::new(bits, width),
		}
	}
}

impl<W: Write> ImageWriter for PngWriter<W> {
	fn write_header(&mut self) -> io::Result<()> {
		let buf = self.buf.as_mut().unwrap();

		let signature = [0x89u8, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
		buf.write_all(&signature)?;

		let header = PngChunk::Ihdr {
			width: self.width as u32,
			height: self.height as u32,
			bit_depth: if self.bits <= 8 { 8 } else { 16 },
		};
		header.write_to(buf)?;

		if self.bits != 8 && self.bits != 16 {
			PngChunk::Sbit(self.bits).write_to(buf)?;
		}
		if let Some(time) = self.time {
			PngChunk::Time(time).write_to(buf)?;
		}
		if let Some(intent) = self.srgb {
			PngChunk::Gama(1.0 / 2.2).write_to(buf)?;
			PngChunk::Srgb(intent).write_to(buf)?;
		}

		Ok(())
	}

	fn write_pixels(&mut self, pixels: &[Color]) -> io::Result<()> {
		if self.pixel_writer.is_none() {
			self.pixel_writer = Some(BufWriter::with_capacity(
				IDAT_SIZE,
				FilterWriter::new(
					ZlibEncoder::new(IdatWriter(self.buf.take().unwrap()), Compression::default()),
					FilterType::None,
					self.width * 3 * if self.bits <= 8 { 1 } else { 2 },
				),
			));
		}
		let pw = self.pixel_writer.as_mut().unwrap();

		for p in pixels {
			let mut p = self.dither.dither(*p);

			let mut written_bits = if self.bits > 8 { 16 } else { 8 };
			p.0 <<= written_bits - self.bits;
			p.1 <<= written_bits - self.bits;
			p.2 <<= written_bits - self.bits;

			// repeat most significant bits into the lower ones so that the overall sample ranges
			// from all zeroes to all ones
			while written_bits > self.bits {
				p.0 |= p.0 >> self.bits;
				p.1 |= p.1 >> self.bits;
				p.2 |= p.2 >> self.bits;
				written_bits -= self.bits;
			}

			if self.bits <= 8 {
				pw.write_all(&[p.0 as u8, p.1 as u8, p.2 as u8])?;
			} else {
				pw.write_all(&[
					(p.0 >> 8) as u8,
					(p.0 & 0xff) as u8,
					(p.1 >> 8) as u8,
					(p.1 & 0xff) as u8,
					(p.2 >> 8) as u8,
					(p.2 & 0xff) as u8,
				])?;
			}
		}
		Ok(())
	}

	fn end(&mut self) -> io::Result<()> {
		let mut buf = if let Some(pw) = self.pixel_writer.take() {
			pw.into_inner()?.finish()?.finish()?.finish()?
		} else {
			self.buf.take().unwrap()
		};
		PngChunk::Iend.write_to(&mut buf)?;
		buf.flush()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_idat_writer() {
		let data: &[u8] = &[
			0x78, 0x9c, 0x63, 0x38, 0xb4, 0x7f, 0xdd, 0xe6, 0xfd, 0x0a, 0x7c, 0x87, 0xe6, 0x70,
			0x6e, 0x52, 0xda, 0x20, 0x34, 0x77, 0x3b, 0x83, 0xa2, 0x30, 0xe3, 0x4a, 0x06, 0x90,
			0xe8, 0x01, 0xb0, 0xe8, 0x66, 0x90, 0xe8, 0x0e, 0xa0, 0x28, 0xc3, 0x4a, 0x86, 0x53,
			0x47, 0x36, 0x6e, 0xda, 0x2f, 0xc9, 0x7e, 0x64, 0x16, 0xc7, 0x2e, 0x85, 0xed, 0x42,
			0x8b, 0xf7, 0x33, 0x28, 0x0b, 0x33, 0xac, 0x67, 0xb0, 0xf3, 0x9f, 0x5d, 0x56, 0x15,
			0x54, 0x92, 0xb6, 0x52, 0xdc, 0xc1, 0xd0, 0xd2, 0xad, 0xd8, 0x99, 0x57, 0xd2, 0xc7,
			0xa5, 0x96, 0x81, 0xc1, 0x28, 0x69, 0xdf, 0xe5, 0x07, 0x3b, 0x7b, 0x4e, 0x6b, 0x32,
			0x44, 0x32, 0x30, 0x32, 0x00, 0x01, 0x0b, 0x2b, 0x03, 0x00, 0x9a, 0xf3, 0x27, 0x9d,
		];
		let mut written: Vec<u8> = Vec::new();
		assert!(IdatWriter(&mut written).write_all(data).is_ok());
		assert_eq!(written.len(), data.len() + 12);
		assert_eq!(
			&written[..8],
			// length, followed by tag
			&[0x00, 0x00, 0x00, 0x70, b'I', b'D', b'A', b'T']
		);
		assert_eq!(&written[8..(data.len() + 8)], data);
		// CRC hash of tag + data
		assert_eq!(&written[(data.len() + 8)..], &[0x62, 0x60, 0x9a, 0xcd]);
	}

	#[test]
	fn test_filter_writer() {
		let data: &[u8] = &[5, 5, 5, 5, 5, 5, 5, 5];
		let mut written: Vec<u8> = Vec::new();
		assert!(FilterWriter::new(&mut written, FilterType::None, 4)
			.write_all(data)
			.is_ok());
		assert_eq!(&written, &[0, 5, 5, 5, 5, 0, 5, 5, 5, 5]);
	}
}
