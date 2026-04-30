use alloc::vec::Vec;

use crate::{Coord, Geometry};

/// Heterogeneous collection of geometries.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct GeometryCollection<T: Coord>(pub Vec<Geometry<T>>);

pub type GeometryCollection2<C = f64> = GeometryCollection<[C; 2]>;
pub type GeometryCollection3<C = f64> = GeometryCollection<[C; 3]>;

/// Read-only abstraction over [`GeometryCollection`].
pub trait GeometryCollectionRef<T: Coord> {
    fn as_slice(&self) -> &[Geometry<T>];

    fn len(&self) -> usize {
        self.as_slice().len()
    }
    fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Geometry<T>> + 'a
    where
        T: 'a,
    {
        self.as_slice().iter()
    }
}

impl<T: Coord> GeometryCollectionRef<T> for GeometryCollection<T> {
    fn as_slice(&self) -> &[Geometry<T>] {
        &self.0
    }
}

impl<T: Coord> GeometryCollection<T> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }
    pub fn from_vec(v: Vec<Geometry<T>>) -> Self {
        Self(v)
    }
    pub fn push(&mut self, g: Geometry<T>) {
        self.0.push(g);
    }
}

impl<T: Coord> From<Vec<Geometry<T>>> for GeometryCollection<T> {
    fn from(v: Vec<Geometry<T>>) -> Self {
        Self(v)
    }
}
