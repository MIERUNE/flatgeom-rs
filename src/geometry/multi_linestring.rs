use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;

use super::{
    Coord,
    linestring::LineStringView,
    offsets::{rebase_offsets, validate_offsets},
};

#[cfg(feature = "serde")]
use super::offsets::{OffsetLengths, offsets_from_lengths};

/// Computer-friendly MultiString
#[derive(Debug, Clone, PartialEq)]
pub struct MultiLineString<T: Coord> {
    /// All coordinates of all LineStrings
    ///
    /// e.g. `[x0, y0, z0, x1, y1, z1, ...]`
    coords: Vec<T>,

    /// A sequence of offsets into coords from which each LineString starts.
    ///
    /// The first offset is always 0, and the last offset is always coords.len().
    line_offsets: Vec<u32>,
}

pub type MultiLineString2<C = f64> = MultiLineString<[C; 2]>;
pub type MultiLineString3<C = f64> = MultiLineString<[C; 3]>;

/// Read-only borrowed view of a MultiLineString.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultiLineStringView<'a, T: Coord> {
    coords: &'a [T],
    line_offsets: &'a [u32],
}

pub type MultiLineStringView2<'a, C = f64> = MultiLineStringView<'a, [C; 2]>;
pub type MultiLineStringView3<'a, C = f64> = MultiLineStringView<'a, [C; 3]>;

/// Read-only abstraction over [`MultiLineString`] and [`MultiLineStringView`].
pub trait MultiLineStringRef<T: Coord> {
    fn coords(&self) -> &[T];
    fn line_offsets(&self) -> &[u32];

    fn raw_coords(&self) -> &[T] {
        let offsets = self.line_offsets();
        match (offsets.first(), offsets.last()) {
            (Some(&start), Some(&end)) => &self.coords()[start as usize..end as usize],
            _ => &[],
        }
    }
    /// Returns line vertex counts, derived as deltas of consecutive line offsets.
    fn line_lengths(&self) -> impl Clone + ExactSizeIterator<Item = u32> + '_ {
        self.line_offsets()
            .windows(2)
            .map(|window| window[1] - window[0])
    }
    fn len(&self) -> usize {
        self.line_offsets().len().saturating_sub(1)
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn iter<'a>(&'a self) -> Iter<'a, T>
    where
        T: 'a,
    {
        Iter {
            coords: self.coords(),
            offsets: self.line_offsets(),
            pos: 0,
            end: self.len(),
        }
    }
    fn iter_range<'a>(&'a self, range: Range<usize>) -> Iter<'a, T>
    where
        T: 'a,
    {
        Iter {
            coords: self.coords(),
            offsets: self.line_offsets(),
            pos: range.start,
            end: range.end,
        }
    }
    fn get(&self, index: usize) -> Option<LineStringView<'_, T>> {
        let offsets = self.line_offsets();
        if index >= self.len() {
            return None;
        }
        Some(LineStringView::from_raw(
            &self.coords()[offsets[index] as usize..offsets[index + 1] as usize],
        ))
    }
}

impl<T: Coord> MultiLineStringRef<T> for MultiLineString<T> {
    fn coords(&self) -> &[T] {
        &self.coords
    }
    fn line_offsets(&self) -> &[u32] {
        &self.line_offsets
    }
}

impl<T: Coord> MultiLineStringRef<T> for MultiLineStringView<'_, T> {
    fn coords(&self) -> &[T] {
        self.coords
    }
    fn line_offsets(&self) -> &[u32] {
        self.line_offsets
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for MultiLineString<T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_multi_line_string(self, serializer)
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for MultiLineStringView<'_, T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_multi_line_string(self, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for MultiLineString<T>
where
    T: Coord + serde::Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct MultiLineStringSerde<T> {
            coords: Vec<T>,
            topology: Vec<u32>,
        }

        let value = MultiLineStringSerde::deserialize(deserializer)?;
        let line_offsets =
            offsets_from_lengths::<D::Error>(&value.topology, value.coords.len(), "topology")?;
        Ok(MultiLineString::from_raw_unchecked(
            value.coords,
            line_offsets,
        ))
    }
}

#[cfg(feature = "serde")]
fn serialize_multi_line_string<T, M, S>(
    multi_line_string: &M,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Coord + serde::Serialize,
    M: MultiLineStringRef<T> + ?Sized,
    S: serde::Serializer,
{
    use serde::ser::SerializeStruct;

    let mut s = serializer.serialize_struct("MultiLineString", 2)?;
    s.serialize_field("coords", multi_line_string.raw_coords())?;
    s.serialize_field("topology", &OffsetLengths(multi_line_string.line_lengths()))?;
    s.end()
}

impl<'a, T: Coord> MultiLineStringView<'a, T> {
    pub const fn from_raw(coords: &'a [T], line_offsets: &'a [u32]) -> Self {
        Self {
            coords,
            line_offsets,
        }
    }

    /// Materializes a self-contained owned `MultiLineString` (rebases
    /// line_offsets to 0-based and copies just this multi's coords).
    pub fn to_owned(&self) -> MultiLineString<T> {
        let (coords, line_offsets) = rebase_offsets(self.coords, self.line_offsets);
        MultiLineString::from_raw_unchecked(coords, line_offsets)
    }
}

impl<T: Coord> Default for MultiLineString<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Coord> MultiLineString<T> {
    /// Creates an empty MultiLineString.
    pub fn new() -> Self {
        Self {
            coords: Vec::new(),
            line_offsets: vec![0],
        }
    }

    pub fn from_raw(coords: Vec<T>, line_offsets: Vec<u32>) -> Self {
        validate_offsets(coords.len(), &line_offsets, "line_offsets");
        Self {
            coords,
            line_offsets,
        }
    }

    pub fn from_raw_unchecked(coords: Vec<T>, line_offsets: Vec<u32>) -> Self {
        Self {
            coords,
            line_offsets,
        }
    }

    /// Removes all linestrings.
    pub fn clear(&mut self) {
        self.coords.clear();
        self.line_offsets.clear();
        self.line_offsets.push(0);
    }

    /// Add a linestring to the MultiLineString.
    pub fn add_linestring<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.coords.extend(iter);
        self.line_offsets.push(self.coords.len() as u32);
    }

    /// Create a new MultiLineString by applying the given transformation to all coordinates.
    pub fn transform<T2: Coord>(&self, f: impl Fn(&T) -> T2) -> MultiLineString<T2> {
        MultiLineString {
            coords: self.coords.iter().map(f).collect(),
            line_offsets: self.line_offsets.clone(),
        }
    }

    /// Applies the given transformation to all coordinates in the MultiLineString.
    pub fn transform_inplace(&mut self, mut f: impl FnMut(&T) -> T) {
        self.coords.iter_mut().for_each(|c| {
            *c = f(c);
        });
    }
}

impl<'a, T: Coord> IntoIterator for &'a MultiLineString<T> {
    type Item = LineStringView<'a, T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        <MultiLineString<T> as MultiLineStringRef<T>>::iter(self)
    }
}

impl<'a, T: Coord> IntoIterator for &MultiLineStringView<'a, T> {
    type Item = LineStringView<'a, T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            coords: self.coords,
            offsets: self.line_offsets,
            pos: 0,
            end: self.line_offsets.len().saturating_sub(1),
        }
    }
}

pub struct Iter<'a, T: Coord> {
    coords: &'a [T],
    offsets: &'a [u32],
    pos: usize,
    end: usize,
}

impl<'a, T: Coord> Iterator for Iter<'a, T> {
    type Item = LineStringView<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.end {
            let start = self.offsets[self.pos] as usize;
            let end = self.offsets[self.pos + 1] as usize;
            self.pos += 1;
            Some(LineStringView::from_raw(&self.coords[start..end]))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.end.saturating_sub(self.pos);
        (n, Some(n))
    }
}

impl<'a, T: Coord> ExactSizeIterator for Iter<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::linestring::LineStringRef;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_mline_basic() {
        let mut mline = MultiLineString2::from_raw(
            (0..7)
                .map(|v| [(v * 2) as f64, (v * 2 + 1) as f64])
                .collect::<Vec<_>>(),
            vec![0, 3, 5, 7],
        );
        assert_eq!(mline.len(), 3);
        assert_eq!(mline.iter().count(), 3);
        for (i, line) in mline.iter().enumerate() {
            match i {
                0 => assert_eq!(line.raw_coords(), &[[0., 1.], [2., 3.], [4., 5.]]),
                1 => assert_eq!(line.raw_coords(), &[[6., 7.], [8., 9.]]),
                2 => assert_eq!(line.raw_coords(), &[[10., 11.], [12., 13.]]),
                _ => unreachable!(),
            }
        }
        mline.clear();
        assert_eq!(mline.len(), 0);
    }

    #[test]
    fn test_mline_one_linestring() {
        let mline = MultiLineString2::from_raw_unchecked(vec![[1., 2.], [3., 4.]], vec![0, 2]);
        assert_eq!(mline.len(), 1);
        assert!(!mline.is_empty());
        for (i, _line) in mline.iter().enumerate() {
            match i {
                0 => assert_eq!(_line.raw_coords(), &[[1., 2.], [3., 4.]]),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_mline_empty() {
        let mline = MultiLineString2::<f64>::new();
        assert_eq!(mline.len(), 0);
        assert!(mline.is_empty());
        assert_eq!(mline.iter().count(), 0);
    }

    #[test]
    fn test_mline_add_linestring() {
        let mut mline = MultiLineString2::new();
        assert_eq!(mline.len(), 0);

        mline.add_linestring(vec![[0., 0.], [1., 1.], [2., 2.]]);
        assert_eq!(mline.len(), 1);

        mline.add_linestring(vec![[3., 3.], [4., 4.], [5., 5.]]);
        assert_eq!(mline.len(), 2);

        mline.add_linestring(vec![[6., 6.], [7., 7.], [8., 8.], [9., 9.]]);
        assert_eq!(mline.len(), 3);

        mline.add_linestring(vec![[1., 1.], [2., 2.], [3., 3.], [4., 4.]]);
        assert_eq!(mline.len(), 4);

        for (i, line) in mline.iter().enumerate() {
            match i {
                0 => assert_eq!(line.raw_coords(), &[[0., 0.], [1., 1.], [2., 2.]]),
                1 => assert_eq!(line.raw_coords(), &[[3., 3.], [4., 4.], [5., 5.]]),
                2 => assert_eq!(line.raw_coords(), &[[6., 6.], [7., 7.], [8., 8.], [9., 9.]]),
                3 => assert_eq!(line.raw_coords(), &[[1., 1.], [2., 2.], [3., 3.], [4., 4.]]),
                _ => unreachable!(),
            }
        }

        for (i, line) in mline.iter_range(1..3).enumerate() {
            match i {
                0 => assert_eq!(line.raw_coords(), &[[3., 3.], [4., 4.], [5., 5.]]),
                1 => assert_eq!(line.raw_coords(), &[[6., 6.], [7., 7.], [8., 8.], [9., 9.]]),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_transform() {
        {
            let mut mlines: MultiLineString2 = MultiLineString2::new();
            mlines.add_linestring([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);
            let new_mlines = mlines.transform(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                new_mlines.iter().next().unwrap().raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }

        {
            let mut mlines = MultiLineString2::new();
            mlines.add_linestring([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);
            mlines.transform_inplace(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                mlines.iter().next().unwrap().raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }
    }

    #[test]
    #[should_panic]
    fn test_mline_invalid_line_offsets_1() {
        let coords: Vec<[f64; 2]> = (0..=2).map(|i| [i as f64, i as f64]).collect();
        let line_offsets: Vec<u32> = vec![0, 99]; // out of `coords` range
        let _polygon: MultiLineString2<f64> = MultiLineString2::from_raw(coords, line_offsets);
    }

    #[test]
    #[should_panic]
    fn test_mline_invalid_line_offsets_2() {
        let coords: Vec<[f64; 2]> = vec![];
        let line_offsets: Vec<u32> = vec![]; // missing sentinel
        let _polygon: MultiLineString2<f64> = MultiLineString2::from_raw(coords, line_offsets);
    }
}
