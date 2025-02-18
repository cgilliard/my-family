#![allow(internal_features)]
#![no_std]
#![feature(unsize)]
#![feature(c_size_t)]
#![feature(coerce_unsized)]
#![feature(core_intrinsics)]
#![no_implicit_prelude]

#[macro_use]
pub mod std;

mod ffi;
pub mod prelude;
mod real_main;

#[cfg(test)]
mod test {
	#[test]
	fn test_mod() {
		assert_eq!(1, 1);
	}
}
