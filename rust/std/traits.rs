use core::mem::size_of;
use core::slice::from_raw_parts;
use prelude::*;
use std::murmur128::murmur3_x64_128_of_slice;

pub trait Display {
	fn format(&self, f: &mut Formatter) -> Result<(), Error>;
}

pub trait Ord {
	fn compare(&self, other: &Self) -> i8;
}

pub trait Hash {
	fn hash(&self) -> usize;
}

impl Ord for i32 {
	fn compare(&self, other: &Self) -> i8 {
		if *self < *other {
			-1
		} else if *self > *other {
			1
		} else {
			0
		}
	}
}

impl Ord for u64 {
	fn compare(&self, other: &Self) -> i8 {
		if *self < *other {
			-1
		} else if *self > *other {
			1
		} else {
			0
		}
	}
}

macro_rules! impl_hash {
	($type:ident) => {
		impl Hash for $type {
			fn hash(&self) -> usize {
				let slice = unsafe {
					from_raw_parts(self as *const $type as *const u8, size_of::<$type>())
				};
				murmur3_x64_128_of_slice(slice, get_murmur_seed()) as usize
			}
		}
	};
}

impl_hash!(u8);
impl_hash!(i8);
impl_hash!(u16);
impl_hash!(i16);
impl_hash!(u32);
impl_hash!(i32);
impl_hash!(u64);
impl_hash!(i64);
impl_hash!(u128);
impl_hash!(i128);
impl_hash!(usize);
impl_hash!(isize);

impl<T> Display for &[T]
where
	T: Display,
{
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		match writeb!(*f, "[") {
			Ok(_) => {}
			Err(e) => return Err(e),
		}
		for i in 0..self.len() {
			match writeb!(*f, "{}", self[i]) {
				Ok(_) => {}
				Err(e) => return Err(e),
			}
			if i < self.len() - 1 {
				match writeb!(*f, ",") {
					Ok(_) => {}
					Err(e) => return Err(e),
				}
			}
		}
		writeb!(*f, "]")
	}
}
