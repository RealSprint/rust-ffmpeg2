use std::ops::{Deref, DerefMut};

use libc::c_int;

use super::Opened;
use crate::{codec::Context, ffi::*, packet, Error};

pub struct Subtitle(pub Opened);

impl Subtitle {
	pub fn decode<P: packet::Ref>(&mut self, packet: &P, out: &mut crate::Subtitle) -> Result<bool, Error> {
		unsafe {
			let mut got: c_int = 0;

			// `avcodec_decode_subtitle2` overwrites the rect array without releasing what is
			// already there, so decoding twice into the same buffer would leak the first result.
			// Freeing first also leaves `out` zeroed on the paths where the decoder produces
			// nothing, rather than holding a stale subtitle.
			avsubtitle_free(out.as_mut_ptr());

			match avcodec_decode_subtitle2(self.as_mut_ptr(), out.as_mut_ptr(), &mut got, packet.as_ptr() as *mut _) {
				e if e < 0 => Err(Error::from(e)),
				_ => Ok(got != 0),
			}
		}
	}
}

impl Deref for Subtitle {
	type Target = Opened;

	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

impl DerefMut for Subtitle {
	fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
		&mut self.0
	}
}

impl AsRef<Context> for Subtitle {
	fn as_ref(&self) -> &Context {
		self
	}
}

impl AsMut<Context> for Subtitle {
	fn as_mut(&mut self) -> &mut Context {
		&mut self.0
	}
}
