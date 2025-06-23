use libc::{c_char, c_int, c_void};

use vsprintf::vsprintf;

use crate::ffi::*;

unsafe extern "C" fn callback(ptr: *mut c_void, level: c_int, fmt: *const c_char, mut args: va_list) {
	if av_log_get_level() <= level {
		return;
	};

	let mut line: &mut [u8] = &mut [0; 1024];
	let mut print_prefix = 1;

	ffmpeg_sys::av_log_format_line(
		ptr,
		level,
		fmt,
		args,
		line.as_mut_ptr() as *mut c_char,
		line.len() as i32,
		&mut print_prefix,
	);

	let Ok(string) = std::str::from_utf8(line.as_ref()) else {
		return;
	};

	let string = string.trim();

	match level {
		AV_LOG_PANIC | AV_LOG_FATAL | AV_LOG_ERROR => tracing::error!("{string}"),
		AV_LOG_WARNING => tracing::warn!("{string}"),
		AV_LOG_INFO => tracing::info!("{string}"),
		AV_LOG_VERBOSE | AV_LOG_DEBUG => tracing::debug!("{string}"),
		AV_LOG_TRACE => tracing::trace!("{string}"),
		_ => {}
	};
}

pub fn register() {
	let asdf: va_list;
	unsafe {
		av_log_set_callback(Some(callback));
	}
}
