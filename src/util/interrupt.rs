use std::{panic, process};

use libc::{c_int, c_void};

use crate::ffi::*;

pub struct Interrupt {
	pub interrupt: AVIOInterruptCB,
}

extern "C" fn callback<F>(opaque: *mut c_void) -> c_int
where
	F: FnMut() -> bool,
{
	let result = panic::catch_unwind(|| unsafe {
		let closure: &mut &mut F = &mut *(opaque as *mut &mut F);
		closure()
	});
	match result {
		Ok(ret) => ret as c_int,
		Err(_) => process::abort(),
	}
}

pub fn new<F>(opaque: Box<F>) -> Interrupt
where
	F: FnMut() -> bool,
{
	let opaque: Box<Box<dyn FnMut() -> bool>> = Box::new(opaque);
	let interrupt_cb = AVIOInterruptCB {
		callback: Some(callback::<F>),
		opaque: Box::into_raw(opaque) as *mut c_void,
	};
	Interrupt {
		interrupt: interrupt_cb,
	}
}
