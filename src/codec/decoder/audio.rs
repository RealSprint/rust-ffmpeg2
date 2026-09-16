use std::ops::{Deref, DerefMut};

use super::Opened;
use crate::{codec::Context, util::format, AudioService, ChannelLayout};

pub struct Audio(pub Opened);

impl Audio {
	pub fn sample_rate(&self) -> u32 {
		unsafe { (*self.as_ptr()).sample_rate as u32 }
	}

	pub fn channels(&self) -> u16 {
		// FFmpeg 7.0 removed the legacy `channels` field; ch_layout is authoritative.
		#[cfg(feature = "ffmpeg_7_0")]
		unsafe {
			(*self.as_ptr()).ch_layout.nb_channels as u16
		}
		#[cfg(not(feature = "ffmpeg_7_0"))]
		unsafe {
			(*self.as_ptr()).channels as u16
		}
	}

	pub fn format(&self) -> format::Sample {
		unsafe { format::Sample::from((*self.as_ptr()).sample_fmt) }
	}

	pub fn request_format(&mut self, value: format::Sample) {
		unsafe {
			(*self.as_mut_ptr()).request_sample_fmt = value.into();
		}
	}

	pub fn frames(&self) -> usize {
		// FFmpeg 7.0 removed AVCodecContext.frame_number in favour of frame_num.
		#[cfg(feature = "ffmpeg_7_0")]
		unsafe {
			(*self.as_ptr()).frame_num as usize
		}
		#[cfg(not(feature = "ffmpeg_7_0"))]
		unsafe {
			(*self.as_ptr()).frame_number as usize
		}
	}

	pub fn align(&self) -> usize {
		unsafe { (*self.as_ptr()).block_align as usize }
	}

	pub fn channel_layout(&self) -> ChannelLayout {
		unsafe { (*self.as_ptr()).ch_layout.into() }
	}

	pub fn set_channel_layout(&mut self, value: ChannelLayout) {
		unsafe {
			(*self.as_mut_ptr()).ch_layout = value.into();
		}
	}

	pub fn audio_service(&mut self) -> AudioService {
		unsafe { AudioService::from((*self.as_mut_ptr()).audio_service_type) }
	}

	pub fn max_bit_rate(&self) -> usize {
		unsafe { (*self.as_ptr()).rc_max_rate as usize }
	}

	pub fn frame_size(&self) -> u32 {
		unsafe { (*self.as_ptr()).frame_size as u32 }
	}
}

impl Deref for Audio {
	type Target = Opened;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Audio {
	fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
		&mut self.0
	}
}

impl AsRef<Context> for Audio {
	fn as_ref(&self) -> &Context {
		self
	}
}

impl AsMut<Context> for Audio {
	fn as_mut(&mut self) -> &mut Context {
		&mut self.0
	}
}
