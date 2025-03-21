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
pub mod net;
pub mod prelude;
mod real_main;
pub mod secp256k1;
pub mod util;
