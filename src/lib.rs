#![no_std]

extern crate alloc;

mod geometry;

#[cfg(feature = "geozero")]
pub mod geozero;

#[cfg(feature = "geo-traits")]
pub mod geo_traits;

pub use geometry::*;
