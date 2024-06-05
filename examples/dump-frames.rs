use core::num;
use std::{collections::HashSet, env, fs::File, io::prelude::*};

use bytes::{Buf, Bytes};
use ffmpeg::{
	format::{input, Pixel},
	media::Type,
	software::scaling::{context::Context, flag::Flags},
	util::frame::video::Video,
	Rational, Rescale,
};

fn main() -> Result<(), ffmpeg::Error> {
	ffmpeg::init().unwrap();

	if let Ok(mut ictx) = input(&env::args().nth(1).expect("Cannot open file.")) {
		let input = ictx.streams().best(Type::Video).ok_or(ffmpeg::Error::StreamNotFound)?;
		let video_stream_index = input.index();

		for (index, res) in ictx.packets().enumerate() {
			let (stream, packet) = res?;

			if stream.id() == 0x13c {
				let mut bytes = Bytes::copy_from_slice(packet.data().unwrap());

				bytes.get_u8();
				bytes.get_u8();
				bytes.get_u8();
				bytes.get_u8();
				bytes.get_u8();
				bytes.get_u8();
				let num_segments = bytes.get_u8();
				if num_segments != 0 {
					for i in 0..num_segments {
						let segment_type = bytes.get_u8();
						let segment_length = bytes.get_u16();
						// println!("{index}: segment_type: {segment_type}, segment_length: {segment_length}",);
						let mut dest = vec![0; segment_length as usize];

						bytes.copy_to_slice(&mut dest);
						let dest = std::str::from_utf8(&dest).unwrap();

						if segment_length > 0 {
							println!(
								"time: {:?} - data {}",
								packet
									.dts()
									.unwrap_or_default()
									.rescale(stream.time_base().unwrap(), Rational(1, 1000)),
								dest
							);
						}
					}
				}
			}
		}
	}

	Ok(())
}

fn save_file(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
	let mut file = File::create(format!("frame{}.ppm", index))?;
	file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
	file.write_all(frame.data(0))?;
	Ok(())
}
