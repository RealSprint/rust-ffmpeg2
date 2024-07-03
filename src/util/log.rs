use libc::{c_char, c_int, c_void};

use vsprintf::vsprintf;

use crate::ffi::*;

#[cfg(target_os = "macos")]
unsafe extern "C" fn callback(_ptr: *mut c_void, level: c_int, fmt: *const c_char, mut args: *mut __va_list_tag) {
	if av_log_get_level() <= level {
		return;
	};

	let string = vsprintf(fmt, args.as_mut_ptr()).unwrap();
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

#[cfg(not(target_os = "macos"))]
unsafe extern "C" fn callback(_ptr: *mut c_void, level: c_int, fmt: *const c_char, mut args: va_list) {
	if av_log_get_level() <= level {
		return;
	};

	let string = vsprintf(fmt, args.as_mut_ptr()).unwrap();
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
	unsafe {
		av_log_set_callback(Some(callback));
	}
}
