pub mod iter;
mod traits;

use std::{
	borrow::Cow,
	ffi::{CStr, CString},
	ptr,
};

use libc::{c_int, c_void};

use self::iter::AVOptionIterator;
pub use self::traits::{Gettable, Iterable, Settable, Target};
use crate::{
	ffi::{AVOptionType::*, *},
	Rational,
};

use super::format::{Pixel, Sample};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Type {
	Flags,
	Int,
	Int64,
	Double,
	Float,
	String,
	Rational,
	Binary,
	Dictionary,
	Constant,

	ImageSize,
	PixelFormat,
	SampleFormat,
	VideoRate,
	Duration,
	Color,
	ChannelLayout,
	c_ulong,
	bool,
}

impl From<AVOptionType> for Type {
	fn from(value: AVOptionType) -> Self {
		match value {
			AV_OPT_TYPE_FLAGS => Type::Flags,
			AV_OPT_TYPE_INT => Type::Int,
			AV_OPT_TYPE_INT64 => Type::Int64,
			AV_OPT_TYPE_DOUBLE => Type::Double,
			AV_OPT_TYPE_FLOAT => Type::Float,
			AV_OPT_TYPE_STRING => Type::String,
			AV_OPT_TYPE_RATIONAL => Type::Rational,
			AV_OPT_TYPE_BINARY => Type::Binary,
			AV_OPT_TYPE_DICT => Type::Dictionary,
			AV_OPT_TYPE_CONST => Type::Constant,
			AV_OPT_TYPE_UINT64 => Type::c_ulong,
			AV_OPT_TYPE_BOOL => Type::bool,

			AV_OPT_TYPE_IMAGE_SIZE => Type::ImageSize,
			AV_OPT_TYPE_PIXEL_FMT => Type::PixelFormat,
			AV_OPT_TYPE_SAMPLE_FMT => Type::SampleFormat,
			AV_OPT_TYPE_VIDEO_RATE => Type::VideoRate,
			AV_OPT_TYPE_DURATION => Type::Duration,
			AV_OPT_TYPE_COLOR => Type::Color,
			// AV_OPT_TYPE_CHANNEL_LAYOUT was removed in FFmpeg 7.0.
			#[cfg(not(feature = "ffmpeg_7_0"))]
			AV_OPT_TYPE_CHANNEL_LAYOUT => Type::ChannelLayout,
			AV_OPT_TYPE_CHLAYOUT => Type::ChannelLayout,
			// AV_OPT_TYPE_UINT was added in FFmpeg 7.1; av_opt_get_int reads it.
			#[cfg(feature = "ffmpeg_7_1")]
			AV_OPT_TYPE_UINT => Type::Int,
			// AV_OPT_TYPE_FLAG_ARRAY (FFmpeg 7.0) only marks another type as an
			// array, which FFmpeg reads and writes whole as a string.
			#[cfg(feature = "ffmpeg_7_0")]
			AV_OPT_TYPE_FLAG_ARRAY => Type::String,
		}
	}
}

impl From<Type> for AVOptionType {
	fn from(val: Type) -> Self {
		match val {
			Type::Flags => AV_OPT_TYPE_FLAGS,
			Type::Int => AV_OPT_TYPE_INT,
			Type::Int64 => AV_OPT_TYPE_INT64,
			Type::Double => AV_OPT_TYPE_DOUBLE,
			Type::Float => AV_OPT_TYPE_FLOAT,
			Type::String => AV_OPT_TYPE_STRING,
			Type::Rational => AV_OPT_TYPE_RATIONAL,
			Type::Binary => AV_OPT_TYPE_BINARY,
			Type::Dictionary => AV_OPT_TYPE_DICT,
			Type::Constant => AV_OPT_TYPE_CONST,
			Type::c_ulong => AV_OPT_TYPE_UINT64,
			Type::bool => AV_OPT_TYPE_BOOL,

			Type::ImageSize => AV_OPT_TYPE_IMAGE_SIZE,
			Type::PixelFormat => AV_OPT_TYPE_PIXEL_FMT,
			Type::SampleFormat => AV_OPT_TYPE_SAMPLE_FMT,
			Type::VideoRate => AV_OPT_TYPE_VIDEO_RATE,
			Type::Duration => AV_OPT_TYPE_DURATION,
			Type::Color => AV_OPT_TYPE_COLOR,
			Type::ChannelLayout => AV_OPT_TYPE_CHLAYOUT,
		}
	}
}

#[derive(Debug)]
pub enum OptionType {
	Int(i64),
	Double(f64),
	String(std::option::Option<Cow<'static, str>>),
	Rational(Rational),
	Bool(bool),
	Pixel(Pixel),
	Sample(Sample),
}

pub struct OptionConstant(Option);

impl OptionConstant {
	pub fn name(&self) -> &str {
		self.0.name()
	}

	pub fn help(&self) -> std::option::Option<&str> {
		self.0.help()
	}

	pub fn default_value(&self) -> OptionType {
		self.0.default_value()
	}
}

pub struct Option {
	obj: *const std::ffi::c_void,
	option: *const AVOption,
	flags: c_int,
}

impl Option {
	pub fn new(obj: *const std::ffi::c_void, option: *const AVOption, flags: c_int) -> Self {
		Self { obj, option, flags }
	}
}

/// The option's type, or `None` for an array option.
///
/// FFmpeg 7.0 ORs `AV_OPT_TYPE_FLAG_ARRAY` into the element type of an array
/// option, a value no `AVOptionType` variant has. `type_` is therefore read as
/// an integer, and neither it nor the whole `AVOption` is read as the enum until
/// that bit is ruled out.
unsafe fn scalar_type(option: *const AVOption) -> std::option::Option<AVOptionType> {
	const _: () = assert!(std::mem::size_of::<AVOptionType>() == std::mem::size_of::<u32>());

	let raw = ptr::addr_of!((*option).type_).cast::<u32>().read();

	#[cfg(feature = "ffmpeg_7_0")]
	if raw & AV_OPT_TYPE_FLAG_ARRAY as u32 != 0 {
		return None;
	}

	Some(std::mem::transmute::<u32, AVOptionType>(raw))
}

impl Option {
	pub fn name(&self) -> &str {
		unsafe { CStr::from_ptr((*self.option).name).to_str().unwrap() }
	}

	pub fn help(&self) -> std::option::Option<&str> {
		if (unsafe { *self.option }).help.is_null() {
			return None;
		}

		Some(unsafe { CStr::from_ptr((*self.option).help).to_str().unwrap() })
	}

	pub fn min(&self) -> f64 {
		unsafe { (*self.option).min }
	}

	pub fn max(&self) -> f64 {
		unsafe { (*self.option).max }
	}

	pub fn kind(&self) -> Type {
		// FFmpeg reads and writes an array option whole as a string.
		unsafe { scalar_type(self.option) }.map_or(Type::String, Type::from)
	}

	/// Whether this is an array option (FFmpeg 7.0+). Its `kind()` is
	/// `Type::String`, the form `av_opt_get` and `av_opt_set` use for the whole
	/// array.
	pub fn is_array(&self) -> bool {
		unsafe { scalar_type(self.option) }.is_none()
	}

	pub fn constants(&self) -> impl Iterator<Item = OptionConstant> + '_ {
		AVOptionIterator::from_option(self.obj, self.option, self.flags)
			.take_while(|option| unsafe { scalar_type(*option) } == Some(AV_OPT_TYPE_CONST))
			.map(|option| OptionConstant(Option::new(self.obj, option, self.flags)))
	}

	pub fn default_value(&self) -> OptionType {
		unsafe {
			let default = (*self.option).default_val;

			let Some(kind) = scalar_type(self.option) else {
				return array_default(self.option);
			};

			match kind {
				AV_OPT_TYPE_FLAGS => OptionType::Int(default.i64_),
				AV_OPT_TYPE_INT
				| AV_OPT_TYPE_INT64
				| AV_OPT_TYPE_UINT64
				| AV_OPT_TYPE_CONST
				| AV_OPT_TYPE_DURATION => OptionType::Int(default.i64_),
				// AV_OPT_TYPE_UINT was added in FFmpeg 7.1.
				#[cfg(feature = "ffmpeg_7_1")]
				AV_OPT_TYPE_UINT => OptionType::Int(default.i64_),
				// AV_OPT_TYPE_CHANNEL_LAYOUT was removed in FFmpeg 7.0.
				#[cfg(not(feature = "ffmpeg_7_0"))]
				AV_OPT_TYPE_CHANNEL_LAYOUT => OptionType::Int(default.i64_),
				AV_OPT_TYPE_DOUBLE | AV_OPT_TYPE_FLOAT => OptionType::Double(default.dbl),
				AV_OPT_TYPE_STRING
				| AV_OPT_TYPE_IMAGE_SIZE
				| AV_OPT_TYPE_DICT
				| AV_OPT_TYPE_BINARY
				| AV_OPT_TYPE_VIDEO_RATE
				| AV_OPT_TYPE_COLOR
				| AV_OPT_TYPE_CHLAYOUT => {
					if default.str_.is_null() {
						return OptionType::String(None);
					}

					OptionType::String(CStr::from_ptr(default.str_).to_str().ok().map(Cow::from))
				}
				#[cfg(feature = "ffmpeg_7_0")]
				AV_OPT_TYPE_FLAG_ARRAY => unreachable!("scalar_type() returns None for array options"),
				AV_OPT_TYPE_BOOL => OptionType::Bool(default.i64_ > 0),
				AV_OPT_TYPE_RATIONAL => OptionType::Rational(default.dbl.into()),
				AV_OPT_TYPE_PIXEL_FMT => OptionType::Pixel(std::mem::transmute::<_, AVPixelFormat>(default.i64_ as i32).into()),
				AV_OPT_TYPE_SAMPLE_FMT => {
					OptionType::Sample(std::mem::transmute::<_, AVSampleFormat>(default.i64_ as i32).into())
				}
			}
		}
	}
}

/// An array option's default: the string `av_opt_set` would parse, held by the
/// `AVOptionArrayDef` the union points to.
#[cfg(feature = "ffmpeg_7_0")]
unsafe fn array_default(option: *const AVOption) -> OptionType {
	let def = (*option).default_val.arr;
	if def.is_null() || (*def).def.is_null() {
		return OptionType::String(None);
	}

	OptionType::String(CStr::from_ptr((*def).def).to_str().ok().map(Cow::from))
}

#[cfg(not(feature = "ffmpeg_7_0"))]
unsafe fn array_default(_: *const AVOption) -> OptionType {
	unreachable!("array options were added in FFmpeg 7.0")
}

pub struct OptionIter {
	inner: AVOptionIterator,
}

impl OptionIter {
	pub fn new(obj: *const std::ffi::c_void, flags: c_int) -> Self {
		Self {
			inner: AVOptionIterator::new(obj, flags),
		}
	}
}

impl Iterator for OptionIter {
	type Item = Option;

	fn next(&mut self) -> std::option::Option<Self::Item> {
		loop {
			let Some(next) = self.inner.next() else {
				return None;
			};

			if unsafe { scalar_type(next) } == Some(AV_OPT_TYPE_CONST) {
				continue;
			}

			return Some(Option::new(self.inner.class(), next, self.inner.flags));
		}
	}
}

pub unsafe fn get_option(ptr: *mut c_void, option: &crate::option::Option) -> std::option::Option<OptionType> {
	let name = CString::new(option.name()).ok()?;

	match option.kind() {
		Type::Flags | Type::Int | Type::Int64 | Type::Constant | Type::Duration | Type::ChannelLayout | Type::c_ulong => {
			let mut value = 0;
			let res = unsafe { av_opt_get_int(ptr, name.as_ptr(), sys::AV_OPT_SEARCH_CHILDREN, &mut value) };

			if res < 0 {
				return None;
			}

			Some(OptionType::Int(value))
		}

		Type::Double | Type::Float => {
			let mut value = 0.0;
			let res = unsafe { av_opt_get_double(ptr, name.as_ptr(), sys::AV_OPT_SEARCH_CHILDREN, &mut value) };

			if res < 0 {
				return None;
			}

			Some(OptionType::Double(value))
		}
		Type::String | Type::Binary | Type::ImageSize | Type::Dictionary | Type::Color | Type::VideoRate => {
			let mut value = ptr::null_mut();
			let res = unsafe { av_opt_get(ptr, name.as_ptr(), sys::AV_OPT_SEARCH_CHILDREN, &mut value) };

			if res < 0 {
				return None;
			}

			if value.is_null() {
				return Some(OptionType::String(None));
			}

			let string = unsafe { CStr::from_ptr(value as *const _) }.to_str().ok()?.to_string();
			unsafe { av_free(value as *mut c_void) };
			Some(OptionType::String(Some(string.into())))
		}
		Type::Rational => {
			let mut value = AVRational { num: 0, den: 1 };
			let res = unsafe { av_opt_get_q(ptr, name.as_ptr(), sys::AV_OPT_SEARCH_CHILDREN, &mut value) };

			if res < 0 {
				return None;
			}

			Some(OptionType::Rational(value.into()))
		}

		Type::PixelFormat => {
			let mut value = AVPixelFormat::AV_PIX_FMT_NONE;
			let res = unsafe { av_opt_get_pixel_fmt(ptr, name.as_ptr(), sys::AV_OPT_SEARCH_CHILDREN, &mut value) };

			if res < 0 {
				return None;
			}

			Some(OptionType::Pixel(value.into()))
		}
		Type::SampleFormat => {
			let mut value = AVSampleFormat::AV_SAMPLE_FMT_NONE;
			let res = unsafe { av_opt_get_sample_fmt(ptr, name.as_ptr(), sys::AV_OPT_SEARCH_CHILDREN, &mut value) };

			if res < 0 {
				return None;
			}

			Some(OptionType::Sample(value.into()))
		}

		Type::bool => {
			let mut value = 0;
			let res = unsafe { av_opt_get_int(ptr, name.as_ptr(), sys::AV_OPT_SEARCH_CHILDREN, &mut value) };

			if res < 0 {
				return None;
			}

			Some(OptionType::Bool(value > 0))
		}
	}
}

#[cfg(all(test, feature = "codec", feature = "ffmpeg_7_0"))]
mod tests {
	use super::{OptionIter, OptionType, Type};
	use crate::ffi::{avcodec_get_class, AV_OPT_FLAG_DECODING_PARAM};

	fn codec_context_option(name: &str) -> super::Option {
		OptionIter::new(unsafe { avcodec_get_class() }.cast(), AV_OPT_FLAG_DECODING_PARAM)
			.find(|option| option.name() == name)
			.unwrap()
	}

	#[test]
	fn array_option_reads_as_a_string() {
		// AV_OPT_TYPE_INT | AV_OPT_TYPE_FLAG_ARRAY on every AVCodecContext since 7.0.
		let option = codec_context_option("side_data_prefer_packet");
		assert!(option.is_array());
		assert_eq!(option.kind(), Type::String);
		assert!(matches!(option.default_value(), OptionType::String(_)));
	}

	#[test]
	fn scalar_option_keeps_its_type() {
		let option = codec_context_option("threads");
		assert!(!option.is_array());
		assert_eq!(option.kind(), Type::Int);
	}
}
