use core::ops::FnOnce;
use core::ptr;
use ffi::{
	release, thread_create, thread_create_joinable, thread_detach, thread_handle_size, thread_join,
};
use prelude::*;

pub struct JoinHandle {
	handle: [u8; 8],
	need_detach: bool,
}

impl Drop for JoinHandle {
	fn drop(&mut self) {
		if self.need_detach {
			let _x = self.detach();
		}
	}
}

impl JoinHandle {
	pub fn join(&mut self) -> Result<(), Error> {
		if !self.need_detach {
			Err(err!(ThreadJoin))
		} else if unsafe { thread_join(&self.handle as *const u8) } != 0 {
			Err(err!(ThreadJoin))
		} else {
			self.need_detach = false;
			Ok(())
		}
	}

	pub fn detach(&mut self) -> Result<(), Error> {
		if !self.need_detach || unsafe { thread_detach(&self.handle as *const u8) } != 0 {
			Err(err!(ThreadDetach))
		} else {
			self.need_detach = false;
			Ok(())
		}
	}
}

extern "C" fn start_thread<F>(ptr: *mut u8)
where
	F: FnOnce(),
{
	let closure = unsafe {
		let mut closure_box = Box::from_raw(Ptr::new(ptr as *mut F));
		closure_box.leak();
		let closure = closure_box.as_ptr().raw() as *mut F;
		let ret = ptr::read(closure);
		release(ptr);
		ret
	};
	closure();
}

pub fn spawn<F>(f: F) -> Result<(), Error>
where
	F: FnOnce(),
{
	match Box::new(f) {
		Ok(mut b) => {
			if unsafe { thread_create(start_thread::<F>, b.as_ptr().raw() as *mut u8) } != 0 {
				return Err(err!(ThreadCreate));
			}
			b.leak();
			Ok(())
		}
		Err(e) => Err(e),
	}
}

pub fn spawnj<F>(f: F) -> Result<JoinHandle, Error>
where
	F: FnOnce(),
{
	if unsafe { thread_handle_size() } > 8 {
		exit!("thread_handle_size() > 8 ({})", unsafe {
			thread_handle_size()
		});
	}
	let jh = JoinHandle {
		handle: [0u8; 8],
		need_detach: true,
	};
	match Box::new(f) {
		Ok(mut b) => {
			if unsafe {
				thread_create_joinable(
					&jh.handle as *const u8,
					start_thread::<F>,
					b.as_ptr().raw() as *mut u8,
				)
			} != 0
			{
				return Err(err!(ThreadCreate));
			}
			b.leak();
			Ok(jh)
		}
		Err(e) => Err(e),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use ffi::{getalloccount, sleep_millis};

	#[test]
	fn test_threads() {
		let initial = unsafe { getalloccount() };
		{
			let lock = lock!();
			let mut x = 1u32;
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(|| {
				let _v = lock.write();
				x += 1;
				assert_eq!(x, 2);
				assert_eq!(*rc_clone, 1);
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			})
			.unwrap();

			loop {
				let _v = lock.write();
				if *rc != 1 {
					assert_eq!(*rc, 2);
					assert_eq!(x, 2);
					break;
				}
			}

			assert!(jh.join().is_ok());
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
	#[test]
	fn test_threads2() {
		let initial = unsafe { getalloccount() };
		{
			let lock = lock!();
			let mut x = 1u32;
			let mut jh = spawnj(|| {
				let _v = lock.write();
				unsafe {
					sleep_millis(50);
				}
				x += 1;
				assert_eq!(x, 2);
			})
			.unwrap();

			loop {
				let _v = lock.write();
				if x != 1 {
					assert_eq!(x, 2);
					break;
				}
			}

			assert!(jh.join().is_ok());
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_thread_join() {
		let initial = unsafe { getalloccount() };
		{
			let lock = lock!();
			let mut x = 1;
			let rc = Rc::new(1).unwrap();
			let mut rc_clone = rc.clone().unwrap();
			let mut jh = spawnj(|| {
				let _v = lock.read(); // memory fence only
				x += 1;
				assert_eq!(x, 2);
				assert_eq!(*rc_clone, 1);
				unsafe {
					sleep_millis(100);
				}
				*rc_clone += 1;
				assert_eq!(*rc_clone, 2);
			})
			.unwrap();

			assert!(jh.join().is_ok());
			assert_eq!(*rc, 2);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
