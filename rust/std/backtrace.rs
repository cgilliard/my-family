use core::ptr::null;
#[cfg(test)]
use core::slice::from_raw_parts;
use ffi::{alloc, backtrace_free, backtrace_ptr, backtrace_size, release};
#[cfg(test)]
use ffi::{backtrace_to_string, cstring_len};
use prelude::*;

#[derive(PartialEq)]
pub struct Backtrace {
	pub(crate) bt: *const u8,
}

impl Drop for Backtrace {
	fn drop(&mut self) {
		unsafe {
			if !self.bt.is_null() {
				backtrace_free(self.bt);
				release(self.bt);
				self.bt = null();
			}
		}
	}
}

impl Backtrace {
	pub fn new() -> Result<Self, Error> {
		let mut bt;
		unsafe {
			bt = alloc(backtrace_size());
			if bt.is_null() {
				return Err(err!(Alloc));
			}
			if backtrace_ptr(bt as *const u8, 100) <= 0 {
				bt = null();
			}
		}
		Ok(Self { bt })
	}

	pub fn print(&self) -> Result<(), Error> {
		match self.to_string() {
			Ok(s) => {
				println!("{}", s);
				Ok(())
			}
			Err(e) => Err(e),
		}
	}

	pub fn to_string(&self) -> Result<String, Error> {
		#[cfg(test)]
		{
			let s = "./bin/test_fam\0";
			let mut txt = null();
			unsafe {
				if !self.bt.is_null() {
					txt = backtrace_to_string(self.bt, s.as_ptr());
				}
			}
			if txt == null() {
				Ok(String::empty())
			} else {
				let len = unsafe { cstring_len(txt) };
				if len == 0 {
					Ok(String::empty())
				} else {
					let bt_slice = unsafe { from_raw_parts(txt, len) };
					let bt_str = unsafe { from_utf8_unchecked(bt_slice) };
					match String::new(bt_str) {
						Ok(backtrace) => Ok(backtrace),
						Err(_) => Ok(String::empty()),
					}
				}
			}
		}
		#[cfg(not(test))]
		{
			String::new("Backtrace: (only in test mode)")
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_bt() {
		let _bt = Backtrace::new().unwrap();
		//_bt.print().unwrap();
	}
}
