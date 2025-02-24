use core::marker::{Copy, Send, Sync};
use core::ptr::write_volatile;
use ffi::cpsrng_rand_bytes_ctx;
use prelude::*;

/// Flag for context to enable no precomputation
pub const SECP256K1_START_NONE: u32 = (1 << 0) | 0;
/// Flag for context to enable verification precomputation
pub const SECP256K1_START_VERIFY: u32 = (1 << 0) | (1 << 8);
/// Flag for context to enable signing precomputation
pub const SECP256K1_START_SIGN: u32 = (1 << 0) | (1 << 9);
/// Flag for keys to indicate uncompressed serialization format
pub const SECP256K1_SER_UNCOMPRESSED: u32 = (1 << 1) | 0;
/// Flag for keys to indicate compressed serialization format
pub const SECP256K1_SER_COMPRESSED: u32 = (1 << 1) | (1 << 8);

/// A nonce generation function. Ordinary users of the library
/// never need to see this type; only if you need to control
/// nonce generation do you need to use it. I have deliberately
/// made this hard to do: you have to write your own wrapper
/// around the FFI functions to use it. And it's an unsafe type.
/// Nonces are generated deterministically by RFC6979 by
/// default; there should be no need to ever change this.
pub type NonceFn = unsafe extern "C" fn(
	nonce32: *mut u8,
	msg32: *const u8,
	key32: *const u8,
	algo16: *const u8,
	attempt: u32,
	data: *const u8,
);

/// A Secp256k1 context, containing various precomputed values and such
/// needed to do elliptic curve computations. If you create one of these
/// with `secp256k1_context_create` you MUST destroy it with
/// `secp256k1_context_destroy`, or else you will have a memory leak.
#[derive(Clone)]
#[repr(C)]
pub struct Context(i32);

/// Secp256k1 aggsig context. As above, needs to be destroyed with
/// `secp256k1_aggsig_context_destroy`
#[derive(Clone)]
#[repr(C)]
pub struct AggSigContext(i32);

/// Secp256k1 scratch space
#[derive(Clone)]
#[repr(C)]
pub struct ScratchSpace(i32);

/// Secp256k1 bulletproof generators
#[derive(Clone)]
#[repr(C)]
pub struct BulletproofGenerators(i32);

/// Generator
#[repr(C)]
#[derive(Clone)]
pub struct Generator(pub [u8; 64]);
impl Copy for Generator {}

/// Library-internal representation of a Secp256k1 public key
#[repr(C)]
#[derive(Clone)]
pub struct PublicKey(pub [u8; 64]);
impl Copy for PublicKey {}

impl PublicKey {
	/// Create a new (zeroed) public key usable for the FFI interface
	pub fn new() -> PublicKey {
		PublicKey([0; 64])
	}
	pub unsafe fn blank() -> Self {
		Self::new()
	}

	pub fn as_mut_ptr(&mut self) -> *mut Self {
		&mut self.0 as *mut u8 as *mut Self
	}

	pub fn as_ptr(&self) -> *const Self {
		&self.0 as *const u8 as *const Self
	}
}

pub const SECRET_KEY_SIZE: usize = 32;
#[repr(C)]
pub struct SecretKey(pub [u8; SECRET_KEY_SIZE]);

impl Drop for SecretKey {
	fn drop(&mut self) {
		for i in 0..SECRET_KEY_SIZE {
			unsafe {
				write_volatile(&mut self.0[i], 0);
			}
		}
	}
}

impl SecretKey {
	pub fn generate(rand: *mut u8) -> Self {
		let mut r = [0u8; 32];
		unsafe { cpsrng_rand_bytes_ctx(rand, &mut r as *mut u8, 32) };
		SecretKey(r)
	}

	pub fn as_mut_ptr(&mut self) -> *mut Self {
		self.0.as_mut_ptr() as *mut Self
	}

	pub fn as_ptr(&self) -> *const Self {
		self.0.as_ptr() as *const Self
	}
}

/// Library-internal representation of a Secp256k1 signature
#[repr(C)]
#[derive(Clone)]
pub struct Signature(pub [u8; 64]);
impl Copy for Signature {}
impl Signature {
	pub fn as_mut_ptr(&mut self) -> *mut Self {
		&mut self.0 as *mut u8 as *mut Self
	}
	pub fn as_ptr(&self) -> *const Self {
		self.0.as_ptr() as *const Self
	}
}

/// Library-internal representation of a Secp256k1 signature + recovery ID
#[repr(C)]
#[derive(Clone)]
pub struct RecoverableSignature([u8; 65]);
impl Copy for RecoverableSignature {}

/// Library-internal representation of a Secp256k1 aggsig partial signature
#[repr(C)]
#[derive(Clone)]
pub struct AggSigPartialSignature([u8; 32]);
impl Copy for AggSigPartialSignature {}

impl Signature {
	/// Create a new (zeroed) signature usable for the FFI interface
	pub fn new() -> Signature {
		Signature([0; 64])
	}
	/// Create a signature from raw data
	pub fn from_data(data: [u8; 64]) -> Signature {
		Signature(data)
	}
	/// Create a new (uninitialized) signature usable for the FFI interface
	pub unsafe fn blank() -> Self {
		Self::new()
	}
}

impl RecoverableSignature {
	/// Create a new (zeroed) signature usable for the FFI interface
	pub fn new() -> RecoverableSignature {
		RecoverableSignature([0; 65])
	}
	/// Create a new (uninitialized) signature usable for the FFI interface
	pub unsafe fn blank() -> Self {
		Self::new()
	}
}

impl AggSigPartialSignature {
	/// Create a new (zeroed) aggsig partial signature usable for the FFI interface
	pub fn new() -> AggSigPartialSignature {
		AggSigPartialSignature([0; 32])
	}
	/// Create a new (uninitialized) signature usable for the FFI interface
	pub unsafe fn blank() -> Self {
		Self::new()
	}
}

/// Library-internal representation of an ECDH shared secret
#[repr(C)]
pub struct SharedSecret([u8; 32]);
impl SharedSecret {
	/// Create a new (zeroed) signature usable for the FFI interface
	pub fn new() -> SharedSecret {
		SharedSecret([0; 32])
	}
	/// Create a new (uninitialized) signature usable for the FFI interface
	pub unsafe fn blank() -> Self {
		Self::new()
	}
}

pub struct Secp256k1 {
	pub(crate) ctx: *mut Context,
	pub(crate) caps: ContextFlag,
}

unsafe impl Send for Secp256k1 {}
unsafe impl Sync for Secp256k1 {}

/// Flags used to determine the capabilities of a `Secp256k1` object;
/// the more capabilities, the more expensive it is to create.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ContextFlag {
	/// Can neither sign nor verify signatures (cheapest to create, useful
	/// for cases not involving signatures, such as creating keys from slices)
	None,
	/// Can sign but not verify signatures
	SignOnly,
	/// Can verify but not create signatures
	VerifyOnly,
	/// Can verify and create signatures
	Full,
	/// Can do all of the above plus pedersen commitments
	Commit,
}

/// The size (in bytes) of a message
pub const MESSAGE_SIZE: usize = 32;

/// A (hashed) message input to an ECDSA signature
#[derive(Clone)]
#[repr(C)]
pub struct Message(pub [u8; MESSAGE_SIZE]);
impl Copy for Message {}
impl Message {
	pub fn as_ptr(&self) -> *const Self {
		self.0.as_ptr() as *const Self
	}
}
