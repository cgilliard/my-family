use prelude::*;

#[must_use = "This `Result` must be used, or explicitly handled with `unwrap`, `is_err`, or similar."]
#[derive(PartialEq)]
pub enum Result<T, E> {
	Ok(T),
	Err(E),
}

impl<T, E> Result<T, E>
where
	E: Display,
{
	pub fn unwrap(self) -> T {
		match self {
			Result::Ok(t) => t,
			Result::Err(e) => exit!("unwrap on error: {}", e),
		}
	}

	pub fn unwrap_err(self) -> E {
		match self {
			Result::Ok(_) => exit!("unwrap_err on ok!"),
			Result::Err(e) => e,
		}
	}

	pub fn is_err(&self) -> bool {
		match self {
			Result::Ok(_) => false,
			_ => true,
		}
	}

	pub fn is_ok(&self) -> bool {
		!self.is_err()
	}
}

#[cfg(test)]
mod test {
	use prelude::*;

	fn test_result() -> Result<(), Error> {
		let x: Result<u32, Error> = Ok(1u32);
		let y = x.unwrap();
		assert_eq!(y, 1);

		Ok(())
	}

	#[test]
	fn call_test_result() {
		match test_result() {
			Ok(_) => {}
			Err(_e) => {
				assert!(false);
			}
		}
	}
}
