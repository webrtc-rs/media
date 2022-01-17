#![warn(rust_2018_idioms)]
#![allow(dead_code)]

pub mod codec;
pub mod device;
pub mod sample;
pub mod track;

mod error;
pub use error::Error;
