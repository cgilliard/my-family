use core::cmp::PartialEq;
use core::iter::{IntoIterator, Iterator};
use core::marker::PhantomData;
use core::mem::{needs_drop, replace, size_of, zeroed};
use core::ops::{Drop, Index, IndexMut, Range};
use core::option::Option as CoreOption;
use core::ptr;
use core::ptr::{copy, copy_nonoverlapping, drop_in_place, null_mut, write_bytes};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use ffi::{alloc, release, resize};
use prelude::*;

pub struct Vec<T> {
	value: Ptr<u8>,
	capacity: usize,
	elements: usize,
	min: usize,
	_marker: PhantomData<T>,
}

impl<T> Clone for Vec<T> {
	fn clone(&self) -> Result<Self, Error> {
		let value_ptr = unsafe { alloc(size_of::<T>() * self.capacity) };
		if value_ptr.is_null() {
			return Err(err!(Alloc));
		}
		let value = Ptr::new(value_ptr);
		Ok(Self {
			value,
			capacity: self.capacity,
			elements: self.elements,
			min: self.min,
			_marker: PhantomData,
		})
	}
}

impl<T: PartialEq> PartialEq for Vec<T> {
	fn eq(&self, other: &Vec<T>) -> bool {
		if self.len() != other.len() {
			false
		} else {
			for i in 0..self.len() {
				if self[i] != other[i] {
					return false;
				}
			}
			true
		}
	}
}

pub struct VecIterator<T> {
	vec: Vec<T>,
	index: usize,
}

impl<T> Iterator for VecIterator<T> {
	type Item = T;

	fn next(&mut self) -> CoreOption<Self::Item> {
		let size = size_of::<T>();
		if self.index < self.vec.elements {
			let ptr = self.vec.value.raw() as *const u8;
			let ptr = unsafe { ptr.add(self.index * size) as *mut T };
			let element = unsafe { replace(&mut *ptr, zeroed()) };
			self.index += 1;
			CoreOption::Some(element)
		} else {
			self.vec.elements = 0;
			CoreOption::None
		}
	}
}

impl<T> IntoIterator for Vec<T> {
	type Item = T;
	type IntoIter = VecIterator<T>;

	fn into_iter(self) -> Self::IntoIter {
		let ret = VecIterator {
			vec: self,
			index: 0,
		};
		ret
	}
}

pub struct VecRefIterator<'a, T> {
	vec: &'a Vec<T>,
	index: usize,
}

impl<'a, T> Iterator for VecRefIterator<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> CoreOption<Self::Item> {
		if self.index < self.vec.len() {
			let item = &self.vec[self.index];
			self.index += 1;
			CoreOption::Some(item)
		} else {
			CoreOption::None
		}
	}
}

impl<'a, T> IntoIterator for &'a Vec<T> {
	type Item = &'a T;
	type IntoIter = VecRefIterator<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		VecRefIterator {
			vec: self,
			index: 0,
		}
	}
}

impl<T> Drop for Vec<T> {
	fn drop(&mut self) {
		if self.value.get_bit() {
			return;
		}
		if needs_drop::<T>() {
			for i in 0..self.elements {
				unsafe {
					let ptr = (self.value.raw() as *const u8).add(i * size_of::<T>()) as *mut T;
					if !self.value.raw().is_null() {
						drop_in_place(ptr);
					}
				}
			}
		}
		let raw = self.value.raw();
		if !raw.is_null() {
			unsafe {
				release(raw);
			}
		}
	}
}

impl<T> Index<usize> for Vec<T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		if index >= self.elements as usize {
			panic!("array index out of bounds!");
		}

		unsafe {
			let target = self.value.raw() as *const T;
			let target = target.add(index);
			&*(target as *const T)
		}
	}
}

impl<T> IndexMut<usize> for Vec<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		if index >= self.elements as usize {
			panic!("array index out of bounds!");
		}

		unsafe {
			let target = self.value.raw() as *const T;
			let target = target.add(index);
			&mut *(target as *mut T)
		}
	}
}

impl<T> Index<Range<usize>> for Vec<T> {
	type Output = [T];

	fn index(&self, range: Range<usize>) -> &Self::Output {
		if range.end > self.len() {
			panic!("Index out of bounds");
		}

		if self.len() == 0 {
			&[]
		} else {
			let element_size = size_of::<T>();
			let start_offset = range.start * element_size;
			unsafe {
				let ptr = (self.value.raw() as *const u8).add(start_offset) as *const T;
				from_raw_parts(ptr, range.end - range.start)
			}
		}
	}
}

impl<T> IndexMut<Range<usize>> for Vec<T> {
	fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
		if range.end > self.len() {
			panic!("Index out of bounds");
		}

		if self.len() == 0 {
			&mut []
		} else {
			let element_size = size_of::<T>();
			let start_offset = range.start * element_size;
			unsafe {
				let ptr = (self.value.raw() as *mut u8).add(start_offset) as *mut T;
				from_raw_parts_mut(ptr, range.end - range.start)
			}
		}
	}
}

impl<T> Vec<T> {
	pub fn new() -> Self {
		Self {
			value: Ptr::new(null_mut()),
			capacity: 0,
			elements: 0,
			min: 16,
			_marker: PhantomData,
		}
	}

	pub fn set_min(&mut self, n: usize) {
		self.min = n;
	}

	pub fn shift(&mut self, n: usize) -> Result<(), Error> {
		if n > self.elements {
			return Err(err!(OutOfBounds));
		}
		self.elements -= n;
		unsafe {
			copy(self.value.raw().add(n), self.value.raw(), self.elements);
		}
		Ok(())
	}

	pub fn push(&mut self, v: T) -> Result<(), Error> {
		let size = size_of::<T>();

		if self.elements + 1 > self.capacity {
			if !self.resize_impl(self.elements + 1) {
				return Err(err!(Alloc));
			}
		}

		let dest_ptr = self.value.raw() as *mut u8;
		unsafe {
			let dest_ptr = dest_ptr.add(size * self.elements) as *mut T;
			ptr::write(dest_ptr, v);
		}
		self.elements += 1;

		Ok(())
	}

	fn next_power_of_two(&self, mut n: usize) -> usize {
		if n < self.min {
			return self.min;
		}
		if n == 0 {
			return 0;
		}
		n -= 1;
		n |= n >> 1;
		n |= n >> 2;
		n |= n >> 4;
		n |= n >> 8;
		n |= n >> 16;
		n |= n >> 32;
		n + 1
	}

	fn resize_impl(&mut self, needed: usize) -> bool {
		let ncapacity = self.next_power_of_two(needed);

		if ncapacity == self.capacity {
			return true;
		}

		let rptr = self.value.raw();

		let nptr = if ncapacity == 0 {
			if !rptr.is_null() {
				unsafe {
					release(rptr as *mut u8);
				}
			}
			null_mut()
		} else if rptr.is_null() {
			unsafe { alloc(ncapacity * size_of::<T>()) }
		} else {
			unsafe { resize(rptr as *mut u8, ncapacity * size_of::<T>()) }
		};
		if !nptr.is_null() {
			if ncapacity > self.capacity {
				let old_size = self.capacity * size_of::<T>();
				let new_size = ncapacity * size_of::<T>();
				unsafe {
					write_bytes((nptr as *mut u8).add(old_size), 0, new_size - old_size);
				}
			}
			self.capacity = ncapacity;
			let nptr = Ptr::new(nptr as *mut u8);
			if self.value.raw().is_null() {
				self.value = nptr;
			} else {
				self.value = nptr;
			}
			true
		} else {
			self.value = Ptr::null();
			ncapacity == 0
		}
	}

	pub fn len(&self) -> usize {
		self.elements
	}

	pub fn clear(&mut self) {
		self.resize_impl(self.min);
		self.elements = 0;
		self.capacity = self.min;
	}

	pub fn as_mut_ptr(&mut self) -> *mut u8 {
		self.value.raw()
	}

	pub fn as_ptr(&self) -> *const u8 {
		self.value.raw()
	}

	pub fn as_slice(&self) -> &[T] {
		unsafe { from_raw_parts(self.value.raw() as *const T, self.elements) }
	}

	pub fn as_mut_slice(&mut self) -> &mut [T] {
		unsafe { from_raw_parts_mut(self.value.raw() as *mut T, self.elements) }
	}

	pub fn resize(&mut self, n: usize) -> Result<(), Error> {
		if self.resize_impl(n) {
			self.elements = n;
			Ok(())
		} else {
			Err(err!(Alloc))
		}
	}

	pub fn append_ptr(&mut self, ptr: *const u8, elems: usize) -> Result<(), Error> {
		if ptr.is_null() {
			return Err(err!(IllegalArgument));
		}
		let size = size_of::<T>();
		let needed = size * (self.elements + elems);
		if needed > self.capacity {
			if !self.resize_impl(needed) {
				return Err(err!(Alloc));
			}
		}

		let dest_ptr = self.value.raw() as *mut u8;
		unsafe {
			let dest_ptr = dest_ptr.add(size * self.elements) as *mut u8;
			copy_nonoverlapping(ptr, dest_ptr, size * elems);
		}

		self.elements += elems;

		Ok(())
	}

	pub fn append(&mut self, v: &Vec<T>) -> Result<(), Error> {
		let size = size_of::<T>();
		let len = v.len();
		let needed = size * (self.elements + len);
		if needed > self.capacity {
			if !self.resize_impl(needed) {
				return Err(err!(Alloc));
			}
		}

		let dest_ptr = self.value.raw() as *mut u8;
		unsafe {
			let dest_ptr = dest_ptr.add(size * len) as *mut u8;
			copy_nonoverlapping(v.value.raw() as *mut u8, dest_ptr, size * len);
		}

		self.elements += len;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use core::fmt::Debug;
	use core::fmt::Error as CoreError;
	use core::fmt::Formatter;
	use core::ops::Drop;
	use core::result::Result as CoreResult;
	use ffi::getalloccount;

	#[test]
	fn test_vec1() {
		let mut x = vec![1, 2, 3, 4, 5, 6].unwrap();
		assert_eq!(x[0], 1);
		assert_eq!(x[1], 2);
		assert_eq!(x[2], 3);
		assert_eq!(x[3], 4);
		assert_eq!(x[4], 5);
		assert_eq!(x[5], 6);
		assert_eq!(x.len(), 6);
		x[5] += 1;
		assert_eq!(x[5], 7);
	}

	#[test]
	fn test_vec2() {
		let initial = unsafe { getalloccount() };
		{
			let mut v1 = Vec::new();
			for i in 0..100000 {
				assert!(v1.push(i).is_ok());
				assert_eq!(v1[i], i);
			}

			for i in 0..100000 {
				v1[i] = i + 100;
			}
			for i in 0..100000 {
				assert_eq!(v1[i], i + 100);
			}

			let v2 = vec![1, 2, 3].unwrap();
			let mut count = 0;
			for x in v2 {
				count += 1;
				assert_eq!(x, count);
			}
			assert_eq!(count, 3);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	impl<T> Debug for Vec<T> {
		fn fmt(&self, _: &mut Formatter<'_>) -> CoreResult<(), CoreError> {
			CoreResult::Ok(())
		}
	}

	#[test]
	fn test_vec_append() {
		let initial = unsafe { getalloccount() };
		{
			let mut v1 = vec![1, 2, 3].unwrap();
			let v2 = vec![4, 5, 6].unwrap();
			assert!(v1.append(&v2).is_ok());

			assert_eq!(v1, vec![1, 2, 3, 4, 5, 6].unwrap());
			assert!(v1 != vec![1, 2, 3, 4, 6, 6].unwrap());
			assert!(v1 == vec![1, 2, 3, 4, 5, 6].unwrap());
			assert!(v1 != v2);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	struct DropTest {
		x: u32,
	}

	static mut VTEST: u32 = 0;

	impl Drop for DropTest {
		fn drop(&mut self) {
			unsafe {
				VTEST += 1;
			}
		}
	}

	#[test]
	fn test_vec_drop() {
		let x = DropTest { x: 8 };

		let initial = unsafe { getalloccount() };
		{
			let mut v: Vec<DropTest> = vec![].unwrap();
			assert!(v.resize(1).is_ok());
			v[0] = x;
			assert_eq!(v[0].x, 8);
		}

		assert_eq!(unsafe { VTEST }, 2);
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_vec_iter_drop() {
		let initial = unsafe { getalloccount() };
		{
			unsafe {
				VTEST = 0;
			}
			{
				let v = vec![DropTest { x: 1 }, DropTest { x: 2 }, DropTest { x: 3 }].unwrap();
				for y in v {
					let _z = y;
				}
			}
			assert_eq!(unsafe { VTEST }, 3);
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}

	#[test]
	fn test_vec_range() {
		let mut v = vec![1, 2, 3, 4, 5].unwrap();
		let r = &v[1..3];
		assert_eq!(r[0], 2);
		assert_eq!(&v[1..3], &vec![2, 3].unwrap()[0..2]);

		let slice = &mut v[1..1];
		assert_eq!(slice, &mut []);

		v.clear();
		assert_eq!(v.len(), 0);
	}

	#[test]
	fn test_set_min0() {
		let initial = unsafe { getalloccount() };
		{
			let mut v = Vec::new();
			v.set_min(0);
			assert!(v.push(1).is_ok());
			assert!(v.resize(128).is_ok());
			assert!(v.resize(0).is_ok());
			// it's already freed at this point
			assert_eq!(initial, unsafe { getalloccount() });
		}
		assert_eq!(initial, unsafe { getalloccount() });
	}
}
