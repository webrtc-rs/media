#![warn(rust_2018_idioms)]
#![allow(dead_code)]

pub mod codec;
mod error;
pub mod io;
pub mod sample;
pub mod track;

pub use error::Error;
