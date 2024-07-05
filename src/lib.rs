#![no_std]

extern crate alloc;

mod geometry;

#[cfg(feature = "geozero")]
pub mod geozero;

pub use geometry::*;
