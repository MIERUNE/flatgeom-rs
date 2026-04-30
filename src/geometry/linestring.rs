use crate::{Coord, Coord2d};
use alloc::vec::Vec;

/// Computer-friendly LineString
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LineString<T: Coord> {
    /// Coordinates of all points
    ///
    /// e.g. `[x0, y0, z0, x1, y1, z1, ...]`
    coords: Vec<T>,
}

pub type LineString2<C = f64> = LineString<[C; 2]>;
pub type LineString3<C = f64> = LineString<[C; 3]>;

/// Read-only borrowed view of a LineString.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineStringView<'a, T: Coord> {
    coords: &'a [T],
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for LineStringView<'_, T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("LineString", 1)?;
        s.serialize_field("coords", self.coords)?;
        s.end()
    }
}

pub type LineStringView2<'a, C = f64> = LineStringView<'a, [C; 2]>;
pub type LineStringView3<'a, C = f64> = LineStringView<'a, [C; 3]>;

/// Read-only abstraction over [`LineString`] and [`LineStringView`].
pub trait LineStringRef<T: Coord> {
    fn raw_coords(&self) -> &[T];

    fn len(&self) -> usize {
        self.raw_coords().len()
    }
    fn is_empty(&self) -> bool {
        self.raw_coords().is_empty()
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a
    where
        T: 'a,
    {
        self.raw_coords().iter().cloned()
    }
    fn iter_closed<'a>(&'a self) -> impl Iterator<Item = T> + 'a
    where
        T: 'a,
    {
        let s = self.raw_coords();
        s.iter().cloned().chain(s.first().cloned())
    }
    /// Returns true if this ring is counter-clockwise.
    fn is_ccw(&self) -> bool
    where
        T: Coord2d,
    {
        self.signed_ring_area() > 0.0
    }
    /// Returns true if this ring is clockwise.
    fn is_cw(&self) -> bool
    where
        T: Coord2d,
    {
        self.signed_ring_area() < 0.0
    }
    /// Calculates the unsigned area of this LineString as a ring.
    fn ring_area(&self) -> f64
    where
        T: Coord2d,
    {
        self.signed_ring_area().abs()
    }
    /// Calculates the signed area of this LineString as a ring.
    fn signed_ring_area(&self) -> f64
    where
        T: Coord2d,
    {
        signed_ring_area(self.raw_coords())
    }
}

impl<T: Coord> LineStringRef<T> for LineString<T> {
    fn raw_coords(&self) -> &[T] {
        &self.coords
    }
}

impl<T: Coord> LineStringRef<T> for LineStringView<'_, T> {
    fn raw_coords(&self) -> &[T] {
        self.coords
    }
}

impl<'a, T: Coord> LineStringView<'a, T> {
    pub const fn from_raw(coords: &'a [T]) -> Self {
        Self { coords }
    }

    /// Materializes a self-contained owned `LineString` from this view.
    pub fn to_owned(&self) -> LineString<T> {
        LineString::from_raw(self.coords.to_vec())
    }
}

impl<'a, T: Coord> IntoIterator for &LineStringView<'a, T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            slice: self.coords,
            pos: 0,
            close: false,
        }
    }
}

impl<T: Coord> LineString<T> {
    /// Creates an empty LineString.
    pub const fn new() -> Self {
        Self { coords: Vec::new() }
    }

    pub fn from_raw(coords: Vec<T>) -> Self {
        Self { coords }
    }

    /// Appends a point to the LineString.
    pub fn push(&mut self, coord: T) {
        self.coords.push(coord);
    }

    /// Extends the LineString with coordinates.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.coords.extend(iter);
    }

    /// Removes all points from the LineString.
    pub fn clear(&mut self) {
        self.coords.clear();
    }

    /// Create a new LineString by applying the given transformation to all coordinates.
    pub fn transform<T2: Coord>(&self, f: impl Fn(&T) -> T2) -> LineString<T2> {
        LineString {
            coords: self.coords.iter().map(f).collect(),
        }
    }

    /// Applies the given transformation to all coordinates in the LineString.
    pub fn transform_inplace(&mut self, mut f: impl FnMut(&T) -> T) {
        self.coords.iter_mut().for_each(|c| {
            *c = f(c);
        });
    }

    /// Reverses the coordinates in the LineString.
    pub fn reverse_inplace(&mut self) {
        self.coords.reverse();
    }

    /// Reverses the winding order of the coordinates in the ring, preserving the first coordinate.
    pub fn reverse_ring_inplace(&mut self) {
        let len = self.coords.len();
        if len > 1 {
            for i in 1..len.div_ceil(2) {
                self.coords.swap(i, len - i);
            }
        }
    }
}

fn signed_ring_area<T: Coord2d>(coords: &[T]) -> f64 {
    if coords.is_empty() {
        return 0.0;
    }
    let mut area = 0.0;
    let mut prev = coords[0].xy();
    for coord in coords.iter().skip(1).chain(coords.first()) {
        let xy = coord.xy();
        area += (prev.0 * xy.1) - (prev.1 * xy.0);
        prev = xy;
    }
    area / 2.0
}

impl<T: Coord> AsRef<[T]> for LineString<T> {
    fn as_ref(&self) -> &[T] {
        &self.coords
    }
}

impl<'a, T: Coord> IntoIterator for &'a LineString<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            slice: &self.coords,
            pos: 0,
            close: false,
        }
    }
}

pub struct Iter<'a, T: Coord> {
    slice: &'a [T],
    pos: usize,
    close: bool,
}

impl<'a, T: Coord> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let v = if self.pos < self.slice.len() {
            Some(self.slice[self.pos])
        } else if self.close && !self.slice.is_empty() && self.pos == self.slice.len() {
            Some(self.slice[0])
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
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_line_basic() {
        let mut line: LineString2 = LineString2::from_raw(
            (0..4)
                .map(|v| [v as f64, v as f64])
                .collect::<Vec<[f64; 2]>>(),
        );
        assert_eq!(line.len(), 4);
        assert!(!line.is_empty());
        for (i, coord) in line.iter().enumerate() {
            match i {
                0 => assert_eq!(coord, [0., 0.]),
                1 => assert_eq!(coord, [1., 1.]),
                2 => assert_eq!(coord, [2., 2.]),
                3 => assert_eq!(coord, [3., 3.]),
                _ => unreachable!(),
            }
        }

        line.clear();
        assert_eq!(line.len(), 0);
        assert!(line.is_empty());

        line.push([0., 1.]);
        assert_eq!(line.len(), 1);
        for _c in &line {}
    }

    #[test]
    fn test_line_empty() {
        let line: LineString2 = LineString2::new();
        assert_eq!(line.len(), 0);
        assert!(line.is_empty());
        assert_eq!(line.iter().count(), 0);
    }

    #[test]
    fn test_line_empty_iter_closed() {
        let line: LineString2 = LineString2::new();
        assert_eq!(line.iter_closed().count(), 0);
    }

    #[test]
    fn test_line_close() {
        let line: LineString2 = LineString2::from_raw(
            (0..3)
                .map(|v| [v as f64, v as f64])
                .collect::<Vec<[f64; 2]>>(),
        );
        assert_eq!(line.len(), 3);
        assert!(!line.is_empty());
        for (i, coord) in line.iter_closed().enumerate() {
            match i {
                0 => assert_eq!(coord, [0., 0.]),
                1 => assert_eq!(coord, [1., 1.]),
                2 => assert_eq!(coord, [2., 2.]),
                3 => assert_eq!(coord, [0., 0.]),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_transform() {
        {
            let line = LineString2::from_raw(vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);
            let new_line = line.transform(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                new_line.raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }

        {
            let mut line = LineString2::from_raw([[0., 0.], [5., 0.], [5., 5.], [0., 5.]].to_vec());
            line.transform_inplace(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(line.raw_coords(), [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]);
        }
    }

    #[test]
    fn test_winding_order() {
        let line = LineString2::from_raw(vec![[0.0, 0.0], [3.0, 0.0], [3.0, 3.0], [0.0, 3.0]]);
        assert!(line.is_ccw());
        assert!(!line.is_cw());

        let line = LineString2::from_raw(vec![[0.0, 0.0], [0.0, 3.0], [3.0, 3.0], [3.0, 0.0]]);
        assert!(!line.is_ccw());
        assert!(line.is_cw());

        let line = LineString2::from_raw(vec![[0.0, 0.0], [0.0, 0.0]]);
        assert!(!line.is_ccw());
        assert!(!line.is_cw());
    }

    #[test]
    fn test_reverse() {
        let mut line = LineString2::from_raw(vec![
            [0.0, 0.0],
            [3.0, 0.0],
            [3.0, 1.0],
            [3.0, 3.0],
            [1.0, 3.0],
            [0.0, 3.0],
        ]);
        line.reverse_inplace();
        assert_eq!(
            line.raw_coords(),
            vec![
                [0.0, 3.0],
                [1.0, 3.0],
                [3.0, 3.0],
                [3.0, 1.0],
                [3.0, 0.0],
                [0.0, 0.0]
            ]
        );

        let mut line = LineString2::from_raw(vec![
            [0.0, 0.0],
            [3.0, 0.0],
            [6.0, 0.0],
            [6.0, 3.0],
            [3.0, 3.0],
        ]);
        line.reverse_inplace();
        assert_eq!(
            line.raw_coords(),
            vec![[3.0, 3.0], [6.0, 3.0], [6.0, 0.0], [3.0, 0.0], [0.0, 0.0]]
        );
    }

    #[test]
    fn test_ring_reverse() {
        let mut line = LineString2::from_raw(vec![
            [0.0, 0.0],
            [3.0, 0.0],
            [3.0, 1.0],
            [3.0, 3.0],
            [1.0, 3.0],
            [0.0, 3.0],
        ]);
        line.reverse_ring_inplace();
        assert_eq!(
            line.raw_coords(),
            vec![
                [0.0, 0.0],
                [0.0, 3.0],
                [1.0, 3.0],
                [3.0, 3.0],
                [3.0, 1.0],
                [3.0, 0.0]
            ]
        );

        let mut line = LineString2::from_raw(vec![
            [0.0, 0.0],
            [3.0, 0.0],
            [6.0, 0.0],
            [6.0, 3.0],
            [3.0, 3.0],
        ]);
        line.reverse_ring_inplace();
        assert_eq!(
            line.raw_coords(),
            vec![[0.0, 0.0], [3.0, 3.0], [6.0, 3.0], [6.0, 0.0], [3.0, 0.0]]
        );
    }
}
