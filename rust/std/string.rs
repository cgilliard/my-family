extern crate core;
use core::cmp::PartialEq;
use core::fmt::Debug;
use core::ptr::copy_nonoverlapping;
use core::slice::from_raw_parts;
use core::str::from_utf8_unchecked;
use prelude::*;
use std::util::strcmp;

pub struct String {
	value: Option<Rc<Box<[u8]>>>,
	end: usize,
	start: usize,
}

impl Display for String {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		f.write_str(self.to_str(), self.len())
	}
}

impl Debug for String {
	fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
		core::result::Result::Ok(())
	}
}

impl PartialEq for String {
	fn eq(&self, other: &String) -> bool {
		strcmp(self.to_str(), other.to_str()) == 0
	}
}

impl Clone for String {
	fn clone(&self) -> Result<Self, Error> {
		match &self.value {
			Some(value) => Ok(Self {
				value: Some(value.clone().unwrap()),
				start: self.start,
				end: self.end,
			}),
			None => Ok(Self::empty()),
		}
	}
}

impl String {
	pub fn new(s: &str) -> Result<Self, Error> {
		let end = s.len();
		let start = 0;
		match Box::new_zeroed_byte_slice(end) {
			Ok(mut value) => {
				let valueptr = value.as_mut_ptr() as *mut u8;
				unsafe {
					copy_nonoverlapping(s.as_ptr(), valueptr, end);
				}
				match Rc::new(value) {
					Ok(rc) => Ok(Self {
						value: Some(rc),
						start,
						end,
					}),
					Err(e) => Err(e),
				}
			}
			Err(e) => Err(e),
		}
	}

	pub fn empty() -> Self {
		Self {
			value: None,
			start: 0,
			end: 0,
		}
	}

	pub fn to_str(&self) -> &str {
		match &self.value {
			Some(value) => {
				let ptr = value.get().as_ptr().raw() as *const u8;
				let ptr = unsafe { ptr.add(self.start) };
				unsafe { from_utf8_unchecked(from_raw_parts(ptr, self.end - self.start)) }
			}
			None => "",
		}
	}

	pub fn substring(&self, start: usize, end: usize) -> Result<Self, Error> {
		if start > end || end - start > self.len() {
			Err(err!(OutOfBounds))
		} else {
			match self.value.clone() {
				Ok(value) => Ok(Self {
					value,
					start: start + self.start,
					end: self.start + end,
				}),
				Err(e) => Err(e),
			}
		}
	}

	pub fn len(&self) -> usize {
		self.end - self.start
	}

	pub fn findn(&self, s: &str, offset: usize) -> Option<usize> {
		let mut x = unsafe { self.to_str().as_ptr().add(offset) };
		let mut len = self.len() as usize;
		let s_len = s.len();

		if s_len == 0 {
			return Some(0);
		}

		unsafe {
			while len >= s_len {
				let v = from_utf8_unchecked(from_raw_parts(x, s_len));
				if strcmp(v, s) == 0 {
					return Some(self.len() as usize - len);
				}
				len -= 1;
				x = x.wrapping_add(1);
			}
		}
		None
	}

	pub fn find(&self, s: &str) -> Option<usize> {
		self.findn(s, 0)
	}

	pub fn rfind(&self, s: &str) -> Option<usize> {
		let s_len = s.len();
		let str_len = self.len() as usize;

		if s_len == 0 {
			return Some(str_len);
		}
		if s_len > str_len {
			return None;
		}

		let mut x = self.to_str().as_ptr().wrapping_add(str_len - s_len);
		let mut len = str_len;

		unsafe {
			while len >= s_len {
				let v = from_utf8_unchecked(from_raw_parts(x, s_len));
				if strcmp(v, s) == 0 {
					return Some(x as usize - self.to_str().as_ptr() as usize);
				}
				len -= 1;
				x = x.wrapping_sub(1);
			}
		}
		None
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use ffi::getalloccount;

	#[test]
	fn test_strings() {
		let initial = unsafe { getalloccount() };
		{
			let x1 = String::new("abcdefghijkl").unwrap();
			assert_eq!(x1.len(), 12);
			assert_eq!(x1.to_str(), "abcdefghijkl");
			assert_eq!(x1.substring(3, 6).unwrap().to_str(), "def");
			let x2 = x1.substring(3, 9).unwrap();
			assert_eq!(x2.to_str(), "defghi");
			assert_eq!(x2, String::new("defghi").unwrap());
			assert_eq!(x1, String::new("abcdefghijkl").unwrap());
			assert_eq!(x1.find("bc"), Some(1));
			assert_eq!(x1.find("aa"), None);
			assert_eq!(x1.find(""), Some(0));
			let x2 = String::new("").unwrap();
			assert_eq!(x2.len(), 0);
			let x3 = String::new("aaabbbcccaaa").unwrap();
			assert_eq!(x3.rfind("aaa"), Some(9));
			assert_eq!(x3.rfind("ajlsfdjklasdjlfalsjkdfjklasdf"), None);
			assert_eq!(x3.rfind("aaaa"), None);
			assert_eq!(x3.find("ajlsfdjklasdjlfalsjkdfjklasdf"), None);
			let x4 = String::new("0123456789012345678901234567890123456789").unwrap();
			assert_eq!(x4.find("012"), Some(0));

			let x5 = x4.clone().unwrap();
			assert_eq!(x5.find("012"), Some(0));
			assert_eq!(x5.rfind("012"), Some(30));

			let x6 = x5.substring(5, 15).unwrap();
			let x7 = x6.to_str().as_bytes();
			assert_eq!(x7.len(), 10);
			assert_eq!(x7[0], b'5');
			let x8 = x5.substring(6, 6).unwrap();
			assert_eq!(x8.len(), 0);

			let x9 = match String::new("test") {
				Ok(s) => s,
				Err(_e) => String::new("").unwrap(),
			};
			assert_eq!(x9.len(), 4);
		}

		assert_eq!(initial, unsafe { getalloccount() });
	}
}
