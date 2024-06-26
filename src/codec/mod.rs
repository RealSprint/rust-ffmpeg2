pub mod flag;
pub use self::flag::Flags;

pub mod id;
pub use self::id::Id;

pub mod packet;

pub mod subtitle;

pub mod discard;

pub mod context;
pub use self::context::Context;

pub mod capabilities;
pub use self::capabilities::Capabilities;

pub mod codec;

#[cfg(feature = "ffmpeg_3_1")]
pub mod parameters;
#[cfg(feature = "ffmpeg_3_1")]
pub use self::parameters::Parameters;

pub mod video;
pub use self::video::Video;

pub mod audio;
pub use self::audio::Audio;

pub mod audio_service;
pub mod field_order;

pub mod compliance;
pub use self::compliance::Compliance;

pub mod debug;
pub use self::debug::Debug;

pub mod profile;
pub use self::profile::Profile;

pub mod threading;

pub mod decoder;
pub use self::decoder::Decoder;
pub mod encoder;
pub use self::encoder::Encoder;
pub mod traits;

pub mod bitstream_filter;

use std::{ffi::CStr, str::from_utf8_unchecked};

use crate::ffi::*;

pub fn version() -> u32 {
	unsafe { avcodec_version() }
}

pub fn configuration() -> &'static str {
	unsafe { from_utf8_unchecked(CStr::from_ptr(avcodec_configuration()).to_bytes()) }
}

pub fn license() -> &'static str {
	unsafe { from_utf8_unchecked(CStr::from_ptr(avcodec_license()).to_bytes()) }
}
