use std::ffi::c_int;

use sys::{av_opt_next, AVClass, AVOption};

pub struct AVOptionIterator {
	obj: *const std::ffi::c_void,
	option: *const AVOption,
	pub(crate) flags: c_int,
}

impl AVOptionIterator {
	pub fn new(av_class: *const std::ffi::c_void, flags: c_int) -> Self {
		Self {
			obj: av_class,
			option: std::ptr::null(),
			flags,
		}
	}

	pub fn from_option(av_class: *const std::ffi::c_void, option: *const AVOption, flags: c_int) -> Self {
		Self {
			obj: av_class,
			option,
			flags,
		}
	}

	pub fn class(&self) -> *const std::ffi::c_void {
		self.obj
	}
}

impl Iterator for AVOptionIterator {
	type Item = *const AVOption;

	fn next(&mut self) -> std::option::Option<<Self as Iterator>::Item> {
		unsafe {
			let priv_class = &self.obj as *const *const std::ffi::c_void;
			let mut ptr = av_opt_next(priv_class as *const std::ffi::c_void, self.option);

			// Skip while the flags aren't set and we haven't reached the end
			while !ptr.is_null() && (*ptr).flags & self.flags == 0 {
				ptr = av_opt_next(priv_class as *const std::ffi::c_void, self.option);
				self.option = ptr;
			}

			if ptr.is_null() {
				None
			} else {
				self.option = ptr;

				Some(ptr)
			}
		}
	}
}
