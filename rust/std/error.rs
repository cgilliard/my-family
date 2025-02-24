use core::ptr::null;
use prelude::*;

macro_rules! define_enum_with_strings {
    ($enum_name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(PartialEq)]
        pub enum $enum_name {
            $($variant),*
        }

        impl $enum_name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)*
                }
            }
        }
    };
}

// Define the enum and string conversion
define_enum_with_strings!(ErrorKind {
	Unknown,
	Alloc,
	OutOfBounds,
	CorruptedData,
	IllegalArgument,
	CapacityExceeded,
	ThreadCreate,
	ThreadJoin,
	InvalidSignature,
	InvalidPublicKey,
	Backtrace,
	ThreadDetach,
	IllegalState,
	Overflow,
	NotInitialized,
	ChannelSend,
	ChannelInit,
	CreateFileDescriptor,
	ConnectionClosed,
	SecpInit,
	SecpErr,
	SecpOddParity,
	WsStop,
	MultiplexRegister,
	SocketConnect,
	Pipe,
	Connect,
	IO,
	Bind,
	InsufficientFunds,
	Todo,
});

#[derive(PartialEq)]
pub struct Error {
	pub kind: ErrorKind,
	pub line: u32,
	pub file: String,
	pub backtrace: Backtrace,
}

impl Error {
	pub fn new(kind: ErrorKind, line: u32, file: &str) -> Self {
		let backtrace;
		#[cfg(test)]
		{
			match Backtrace::new() {
				Ok(bt) => {
					backtrace = bt;
				}
				Err(_) => {
					backtrace = Backtrace { bt: null() };
				}
			}
		}
		#[cfg(not(test))]
		{
			backtrace = Backtrace { bt: null() };
		}
		Self {
			backtrace,
			kind,
			line,
			file: match String::new(file) {
				Ok(file) => file,
				Err(_) => String::empty(),
			},
		}
	}
}

impl Display for Error {
	fn format(&self, f: &mut Formatter) -> Result<(), Error> {
		match writeb!(
			*f,
			"Error[kind={},loc={}:{}]\n",
			self.kind.as_str(),
			self.file,
			self.line
		) {
			Ok(_) => match self.backtrace.to_string() {
				Ok(bt) => writeb!(*f, "{}", bt),
				Err(_) => Ok(()),
			},
			Err(e) => Err(e),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_err() {
		let _x = err!(Alloc);
		//println!("x=\n'{}'", _x);
	}
}
