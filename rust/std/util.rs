use core::intrinsics::{unchecked_div, unchecked_rem};
use core::mem::size_of;
use core::ptr::copy_nonoverlapping;
use core::slice::from_raw_parts;
use ffi::{rand_bytes, sleep_millis};
use prelude::*;

pub fn copy_from_slice(dst: &mut [u8], src: &[u8]) {
	let len = src.len();
	if len > dst.len() {
		exit!(
			"copy_from_slice: src.len() must be less than or equal to dst.len() [{} / {}]",
			src.len(),
			dst.len()
		);
	}

	unsafe {
		copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), len);
	}
}

pub fn copy_from_slice_u64(dst: &mut [u64], src: &[u64]) {
	let len = src.len();
	if len > dst.len() {
		exit!(
			"copy_from_slice: src.len() must be less than or equal to dst.len() [{} / {}]",
			src.len(),
			dst.len()
		);
	}

	unsafe {
		copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), len);
	}
}

pub fn subslice<N>(n: &[N], off: usize, len: usize) -> Result<&[N], Error> {
	if len + off > n.len() {
		Err(err!(OutOfBounds))
	} else {
		Ok(unsafe { from_raw_parts(n.as_ptr().add(off), len) })
	}
}

pub fn to_be_bytes_u64(value: u64, bytes: &mut [u8]) {
	if bytes.len() >= 8 {
		bytes[0] = (value >> 56) as u8;
		bytes[1] = (value >> 48) as u8;
		bytes[2] = (value >> 40) as u8;
		bytes[3] = (value >> 32) as u8;
		bytes[4] = (value >> 24) as u8;
		bytes[5] = (value >> 16) as u8;
		bytes[6] = (value >> 8) as u8;
		bytes[7] = value as u8;
	}
}

pub fn to_be_bytes_u32(value: u32, bytes: &mut [u8]) {
	if bytes.len() >= 4 {
		bytes[0] = (value >> 24) as u8;
		bytes[1] = (value >> 16) as u8;
		bytes[2] = (value >> 8) as u8;
		bytes[3] = value as u8;
	}
}

pub fn to_be_bytes_u16(value: u16, bytes: &mut [u8]) {
	if bytes.len() >= 2 {
		bytes[0] = (value >> 8) as u8;
		bytes[1] = value as u8;
	}
}

pub fn to_be_bytes_u128(value: u128, bytes: &mut [u8]) {
	if bytes.len() >= 16 {
		bytes[0] = (value >> 120) as u8;
		bytes[1] = (value >> 112) as u8;
		bytes[2] = (value >> 104) as u8;
		bytes[3] = (value >> 96) as u8;
		bytes[4] = (value >> 88) as u8;
		bytes[5] = (value >> 80) as u8;
		bytes[6] = (value >> 72) as u8;
		bytes[7] = (value >> 64) as u8;
		bytes[8] = (value >> 56) as u8;
		bytes[9] = (value >> 48) as u8;
		bytes[10] = (value >> 40) as u8;
		bytes[11] = (value >> 32) as u8;
		bytes[12] = (value >> 24) as u8;
		bytes[13] = (value >> 16) as u8;
		bytes[14] = (value >> 8) as u8;
		bytes[15] = value as u8;
	}
}

pub fn from_be_bytes_u64(bytes: &[u8]) -> u64 {
	if bytes.len() >= 8 {
		((bytes[0] as u64) << 56)
			| ((bytes[1] as u64) << 48)
			| ((bytes[2] as u64) << 40)
			| ((bytes[3] as u64) << 32)
			| ((bytes[4] as u64) << 24)
			| ((bytes[5] as u64) << 16)
			| ((bytes[6] as u64) << 8)
			| (bytes[7] as u64)
	} else {
		0
	}
}

pub fn from_be_bytes_u16(bytes: &[u8]) -> u16 {
	if bytes.len() >= 2 {
		((bytes[0] as u16) << 8) | (bytes[1] as u16)
	} else {
		0
	}
}

pub fn to_le_bytes_u64(value: u64, bytes: &mut [u8]) {
	if bytes.len() >= 8 {
		bytes[0] = value as u8;
		bytes[1] = (value >> 8) as u8;
		bytes[2] = (value >> 16) as u8;
		bytes[3] = (value >> 24) as u8;
		bytes[4] = (value >> 32) as u8;
		bytes[5] = (value >> 40) as u8;
		bytes[6] = (value >> 48) as u8;
		bytes[7] = (value >> 56) as u8;
	}
}

pub fn to_le_bytes_u16(value: u16, bytes: &mut [u8]) {
	if bytes.len() >= 2 {
		bytes[0] = value as u8;
		bytes[1] = (value >> 8) as u8;
	}
}

pub fn from_le_bytes_u64(bytes: &[u8]) -> u64 {
	if bytes.len() >= 8 {
		(bytes[0] as u64)
			| ((bytes[1] as u64) << 8)
			| ((bytes[2] as u64) << 16)
			| ((bytes[3] as u64) << 24)
			| ((bytes[4] as u64) << 32)
			| ((bytes[5] as u64) << 40)
			| ((bytes[6] as u64) << 48)
			| ((bytes[7] as u64) << 56)
	} else {
		0
	}
}

pub fn from_le_bytes_u16(bytes: &[u8]) -> u16 {
	if bytes.len() >= 2 {
		(bytes[0] as u16) | ((bytes[1] as u16) << 8)
	} else {
		0
	}
}

pub fn to_le_bytes_u32(value: u32, bytes: &mut [u8]) {
	if bytes.len() >= 4 {
		bytes[0] = value as u8;
		bytes[1] = (value >> 8) as u8;
		bytes[2] = (value >> 16) as u8;
		bytes[3] = (value >> 24) as u8;
	}
}

pub fn from_le_bytes_u32(bytes: &[u8]) -> u32 {
	if bytes.len() >= 4 {
		(bytes[0] as u32)
			| ((bytes[1] as u32) << 8)
			| ((bytes[2] as u32) << 16)
			| ((bytes[3] as u32) << 24)
	} else {
		0
	}
}

pub fn from_be_bytes_u32(bytes: &[u8]) -> u32 {
	if bytes.len() >= 4 {
		((bytes[0] as u32) << 24)
			| ((bytes[1] as u32) << 16)
			| ((bytes[2] as u32) << 8)
			| (bytes[3] as u32)
	} else {
		0
	}
}

pub fn u128_to_str(mut n: u128, offset: usize, buf: &mut [u8], base: u8) -> usize {
	let buf_len = buf.len();
	let mut i = buf_len - 1;

	while n > 0 {
		if i == 0 {
			break;
		}
		if i < buf_len && base != 0 {
			let digit = (n % base as u128) as u8;
			buf[i] = if digit < 10 {
				b'0' + digit
			} else {
				b'a' + (digit - 10)
			};
		}
		if base != 0 {
			n /= base as u128;
		}
		i -= 1;
	}

	let mut len = buf_len - i - 1;

	if len == 0 && buf_len > 0 && offset < buf_len {
		buf[offset] = b'0';
		len = 1;
	} else {
		let mut k = 0;
		for j in i + 1..buf_len {
			if k + offset < buf_len {
				buf[k + offset] = buf[j];
			}
			k += 1;
		}
	}

	len
}

pub fn i128_to_str(mut n: i128, buf: &mut [u8], base: u8) -> usize {
	if n < 0 {
		n *= -1;
		if buf.len() < 2 {
			0
		} else {
			buf[0] = b'-';
			u128_to_str(n as u128, 1, buf, base) + 1
		}
	} else {
		u128_to_str(n as u128, 0, buf, base)
	}
}

pub fn strcmp(a: &str, b: &str) -> i32 {
	let len = if a.len() > b.len() { b.len() } else { a.len() };
	let x = a.as_bytes();
	let y = b.as_bytes();

	for i in 0..len {
		if x[i] != y[i] {
			return if x[i] > y[i] { 1 } else { -1 };
		}
	}

	if a.len() < b.len() {
		1
	} else if a.len() > b.len() {
		-1
	} else {
		0
	}
}

pub fn copy_slice(src: &[u8], dest: &mut [u8], len: usize) {
	unsafe {
		copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), len);
	}
}

pub fn divide_usize(n: usize, d: usize) -> usize {
	if d == 0 {
		//exit!("divide by 0!");
	}
	unsafe { unchecked_div(n, d) }
}

pub fn rem_usize(n: usize, d: usize) -> usize {
	if d == 0 {
		//exit!("rem by 0!");
	}
	unsafe { unchecked_rem(n, d) }
}

pub fn park() {
	loop {
		unsafe {
			sleep_millis(1000 * 60);
		}
	}
}

static mut STATIC_MURMUR_SEED: u64 = 0u64;

#[allow(static_mut_refs)]
pub fn get_murmur_seed() -> u32 {
	unsafe {
		loop {
			let cur = aload!(&STATIC_MURMUR_SEED);
			if cur != 0 {
				return cur as u32;
			}
			let mut nval = 0u64;
			rand_bytes(&mut nval as *mut u64 as *mut u8, size_of::<u64>());
			if nval == 0 {
				continue;
			}
			if cas!(&mut STATIC_MURMUR_SEED, &cur, nval) {
				return nval as u32;
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::strcmp;

	#[test]
	fn test_strcmp() {
		assert_eq!(strcmp("abc", "abc"), 0);
		assert_eq!(strcmp("abc", "def"), -1);
		assert_eq!(strcmp("def", "abc"), 1);
	}
}
