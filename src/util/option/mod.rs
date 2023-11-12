pub mod iter;
mod traits;

use std::ffi::CStr;

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
			AV_OPT_TYPE_CHANNEL_LAYOUT => Type::ChannelLayout,
			AV_OPT_TYPE_CHLAYOUT => Type::ChannelLayout,
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
	String(std::option::Option<&'static str>),
	Rational(Rational),
	Bool(bool),
	Pixel(Pixel),
	Sample(Sample),
}

pub struct OptionConstants(Option);

impl OptionConstants {
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
	class: *const AVClass,
	option: *const AVOption,
}

impl Option {
	pub fn new(class: *const AVClass, option: *const AVOption) -> Self {
		Self { class, option }
	}
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
		unsafe { (*self.option).type_.into() }
	}

	pub fn constants(&self) -> impl Iterator<Item = OptionConstants> + '_ {
		AVOptionIterator::from_option(self.class, self.option)
			.take_while(|option| {
				let option = *option;
				(unsafe { *option }).type_ == AV_OPT_TYPE_CONST
			})
			.map(|option| OptionConstants(Option::new(self.class, option)))
	}

	pub fn default_value(&self) -> OptionType {
		unsafe {
			let default = (*self.option).default_val;

			match ({ *self.option }).type_ {
				AV_OPT_TYPE_FLAGS => OptionType::Int(default.i64_),
				AV_OPT_TYPE_INT
				| AV_OPT_TYPE_INT64
				| AV_OPT_TYPE_UINT64
				| AV_OPT_TYPE_CONST
				| AV_OPT_TYPE_DURATION
				| AV_OPT_TYPE_CHANNEL_LAYOUT => OptionType::Int(default.i64_),
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

					OptionType::String(Some(CStr::from_ptr(default.str_).to_str().unwrap()))
				}
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

pub struct OptionIter {
	inner: AVOptionIterator,
}

impl OptionIter {
	pub fn new(class: *const AVClass) -> Self {
		Self {
			inner: AVOptionIterator::new(class),
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

			if (unsafe { *next }).type_ == AV_OPT_TYPE_CONST {
				continue;
			}

			return Some(Option::new(self.inner.class(), next));
		}
	}
}
