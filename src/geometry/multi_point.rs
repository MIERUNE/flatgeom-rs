use crate::Coord;
use alloc::vec::Vec;
use core::ops::Range;

/// Computer-friendly MultiPoint
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MultiPoint<T: Coord> {
    /// Coordinates of all points
    ///
    /// e.g. `[x0, y0, z0, x1, y1, z1, ...]`
    coords: Vec<T>,
}

pub type MultiPoint2<C = f64> = MultiPoint<[C; 2]>;
pub type MultiPoint3<C = f64> = MultiPoint<[C; 3]>;

/// Read-only borrowed view of a MultiPoint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultiPointView<'a, T: Coord> {
    coords: &'a [T],
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for MultiPointView<'_, T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("MultiPoint", 1)?;
        s.serialize_field("coords", self.coords)?;
        s.end()
    }
}

pub type MultiPointView2<'a, C = f64> = MultiPointView<'a, [C; 2]>;
pub type MultiPointView3<'a, C = f64> = MultiPointView<'a, [C; 3]>;

/// Read-only abstraction over [`MultiPoint`] and [`MultiPointView`].
pub trait MultiPointRef<T: Coord> {
    fn raw_coords(&self) -> &[T];

    fn len(&self) -> usize {
        self.raw_coords().len()
    }
    fn is_empty(&self) -> bool {
        self.raw_coords().is_empty()
    }
    fn get(&self, index: usize) -> Option<T> {
        self.raw_coords().get(index).cloned()
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a
    where
        T: 'a,
    {
        self.raw_coords().iter().cloned()
    }
    fn iter_range<'a>(&'a self, range: Range<usize>) -> Iter<'a, T>
    where
        T: 'a,
    {
        Iter {
            slice: self.raw_coords(),
            pos: range.start,
            end: range.end,
        }
    }
}

impl<T: Coord> MultiPointRef<T> for MultiPoint<T> {
    fn raw_coords(&self) -> &[T] {
        &self.coords
    }
}

impl<T: Coord> MultiPointRef<T> for MultiPointView<'_, T> {
    fn raw_coords(&self) -> &[T] {
        self.coords
    }
}

impl<'a, T: Coord> MultiPointView<'a, T> {
    pub const fn from_raw(coords: &'a [T]) -> Self {
        Self { coords }
    }

    /// Materializes a self-contained owned `MultiPoint` from this view.
    pub fn to_owned(&self) -> MultiPoint<T> {
        MultiPoint::from_raw(self.coords.to_vec())
    }
}

impl<'a, T: Coord> IntoIterator for &MultiPointView<'a, T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            slice: self.coords,
            pos: 0,
            end: self.coords.len(),
        }
    }
}

impl<T: Coord> MultiPoint<T> {
    /// Creates an empty MultiPoint.
    pub const fn new() -> Self {
        Self { coords: Vec::new() }
    }

    pub fn from_raw(coords: Vec<T>) -> Self {
        Self { coords }
    }

    /// Appends a coordinate to the MultiPoint.
    pub fn push(&mut self, coord: T) {
        self.coords.push(coord);
    }

    /// Extends the MultiPoint with coordinates.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.coords.extend(iter);
    }

    /// Removes all points from the MultiPoint.
    pub fn clear(&mut self) {
        self.coords.clear();
    }

    /// Create a new MultiPoint by applying the given transformation to all coordinates.
    pub fn transform<T2: Coord>(&self, f: impl Fn(&T) -> T2) -> MultiPoint<T2> {
        MultiPoint {
            coords: self.coords.iter().map(f).collect(),
        }
    }

    /// Applies the given transformation to all coordinates in the MultiPoint.
    pub fn transform_inplace(&mut self, mut f: impl FnMut(&T) -> T) {
        self.coords.iter_mut().for_each(|c| {
            *c = f(c);
        });
    }
}

impl<T: Coord> AsRef<[T]> for MultiPoint<T> {
    fn as_ref(&self) -> &[T] {
        &self.coords
    }
}

impl<'a, T: Coord> IntoIterator for &'a MultiPoint<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            slice: &self.coords,
            pos: 0,
            end: self.coords.len(),
        }
    }
}

pub struct Iter<'a, T: Coord> {
    slice: &'a [T],
    pos: usize,
    end: usize,
}

impl<'a, T: Coord> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let v = if self.pos < self.end {
            Some(self.slice[self.pos])
        } else {
            None
        };
        self.pos += 1;
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpoints_basic() {
        let mut mpoints = MultiPoint2::new();
        assert!(mpoints.is_empty());
        assert_eq!(mpoints.len(), 0);

        mpoints.push([0.0, 0.0]);
        assert!(!mpoints.is_empty());
        assert_eq!(mpoints.len(), 1);

        mpoints.push([1.0, 1.0]);
        mpoints.push([2.0, 2.0]);
        mpoints.push([3.0, 3.0]);
        assert_eq!(mpoints.get(0), Some([0.0, 0.0]));
        assert_eq!(mpoints.get(1), Some([1.0, 1.0]));
        assert_eq!(mpoints.get(2), Some([2.0, 2.0]));
        assert_eq!(mpoints.get(3), Some([3.0, 3.0]));
        assert_eq!(mpoints.len(), 4);
        assert_eq!(mpoints.iter().count(), 4);
        assert_eq!((&mpoints).into_iter().count(), 4);

        for (i, point) in mpoints.iter().enumerate() {
            match i {
                0 => assert_eq!(point, [0.0, 0.0]),
                1 => assert_eq!(point, [1.0, 1.0]),
                2 => assert_eq!(point, [2.0, 2.0]),
                3 => assert_eq!(point, [3.0, 3.0]),
                _ => unreachable!(),
            }
        }

        for (i, point) in mpoints.iter_range(1..3).enumerate() {
            match i {
                0 => assert_eq!(point, [1.0, 1.0]),
                1 => assert_eq!(point, [2.0, 2.0]),
                _ => unreachable!(),
            }
        }

        mpoints.clear();
        assert!(mpoints.is_empty());
        assert_eq!(mpoints.len(), 0);
        assert_eq!(mpoints.iter().count(), 0);
    }

    #[test]
    fn test_mpoints_from_raw() {
        let mpoints = MultiPoint2::from_raw([[1.2, 2.1], [3.4, 4.3]].to_vec());
        assert_eq!(mpoints.as_ref(), [[1.2, 2.1], [3.4, 4.3]]);
        assert_eq!(mpoints.raw_coords(), [[1.2, 2.1], [3.4, 4.3]]);
    }

    #[test]
    fn test_transform() {
        {
            let mpoints = MultiPoint2::from_raw([[0., 0.], [5., 0.], [5., 5.], [0., 5.]].to_vec());
            let new_mpoints = mpoints.transform(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                new_mpoints.raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }

        {
            let mut mpoints =
                MultiPoint2::from_raw([[0., 0.], [5., 0.], [5., 5.], [0., 5.]].to_vec());
            mpoints.transform_inplace(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                mpoints.raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }
    }
}
