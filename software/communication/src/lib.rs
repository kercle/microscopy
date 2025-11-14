#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod uart;

#[cfg(feature = "std")]
pub mod ws;
