use crate::Coord2d;
use alloc::vec;
use alloc::vec::Vec;

use super::{
    Coord,
    linestring::{LineStringRef, LineStringView},
    offsets::{extend_ring_dedup_closing, rebase_offsets, validate_offsets},
};

#[cfg(feature = "serde")]
use super::offsets::{OffsetLengths, offsets_from_lengths};

/// Computer-friendly Polygon
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon<T: Coord> {
    /// Coordinates
    coords: Vec<T>,

    /// A sequence of offsets into coords from which each ring starts.
    ///
    /// The first offset is always 0, and the last offset is always coords.len().
    ring_offsets: Vec<u32>,
}

pub type Polygon3<C = f64> = Polygon<[C; 3]>;
pub type Polygon2<C = f64> = Polygon<[C; 2]>;

/// Read-only borrowed view of a Polygon (may use absolute offsets into a parent buffer).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolygonView<'a, T: Coord> {
    coords: &'a [T],
    ring_offsets: &'a [u32],
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for Polygon<T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_polygon(self, serializer)
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for PolygonView<'_, T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_polygon(self, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Polygon<T>
where
    T: Coord + serde::Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct PolygonSerde<T> {
            coords: Vec<T>,
            topology: Vec<u32>,
        }

        let value = PolygonSerde::deserialize(deserializer)?;
        let ring_offsets =
            offsets_from_lengths::<D::Error>(&value.topology, value.coords.len(), "topology")?;
        Ok(Polygon::from_raw_unchecked(value.coords, ring_offsets))
    }
}

#[cfg(feature = "serde")]
fn serialize_polygon<T, P, S>(polygon: &P, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Coord + serde::Serialize,
    P: PolygonRef<T> + ?Sized,
    S: serde::Serializer,
{
    use serde::ser::SerializeStruct;

    let mut s = serializer.serialize_struct("Polygon", 2)?;
    s.serialize_field("coords", polygon.raw_coords())?;
    s.serialize_field("topology", &OffsetLengths(polygon.ring_lengths()))?;
    s.end()
}

pub type PolygonView2<'a, C = f64> = PolygonView<'a, [C; 2]>;
pub type PolygonView3<'a, C = f64> = PolygonView<'a, [C; 3]>;

/// Read-only abstraction over [`Polygon`] and [`PolygonView`].
///
/// Default impls assume `coords` and `ring_offsets` are mutually consistent —
/// i.e. `coords[ring_offsets[i]..ring_offsets[i + 1]]` yields ring `i`. This
/// holds whether `ring_offsets` are zero-based (owned `Polygon`) or absolute
/// into a parent buffer (`PolygonView` from `MultiPolygon`).
pub trait PolygonRef<T: Coord> {
    fn coords(&self) -> &[T];
    fn ring_offsets(&self) -> &[u32];

    fn len(&self) -> usize {
        self.ring_offsets().len().saturating_sub(1)
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns only this polygon's coordinates (sliced from the underlying buffer).
    fn raw_coords(&self) -> &[T] {
        let r = self.ring_offsets();
        match (r.first(), r.last()) {
            (Some(&s), Some(&e)) => &self.coords()[s as usize..e as usize],
            _ => &[],
        }
    }
    /// Returns ring vertex counts, derived as deltas of consecutive ring offsets.
    fn ring_lengths(&self) -> impl Clone + ExactSizeIterator<Item = u32> + '_ {
        self.ring_offsets()
            .windows(2)
            .map(|window| window[1] - window[0])
    }
    fn exterior(&self) -> LineStringView<'_, T> {
        if self.is_empty() {
            LineStringView::from_raw(&[])
        } else {
            let r = self.ring_offsets();
            LineStringView::from_raw(&self.coords()[r[0] as usize..r[1] as usize])
        }
    }
    fn interiors<'a>(&'a self) -> impl Iterator<Item = LineStringView<'a, T>> + 'a
    where
        T: 'a,
    {
        let r = self.ring_offsets();
        let coords = self.coords();
        let len = self.len();
        (1..len).map(move |i| LineStringView::from_raw(&coords[r[i] as usize..r[i + 1] as usize]))
    }
    fn rings<'a>(&'a self) -> impl Iterator<Item = LineStringView<'a, T>> + 'a
    where
        T: 'a,
    {
        let r = self.ring_offsets();
        let coords = self.coords();
        let len = self.len();
        (0..len).map(move |i| LineStringView::from_raw(&coords[r[i] as usize..r[i + 1] as usize]))
    }
    fn ring(&self, index: usize) -> Option<LineStringView<'_, T>> {
        let r = self.ring_offsets();
        if index >= self.len() {
            return None;
        }
        Some(LineStringView::from_raw(
            &self.coords()[r[index] as usize..r[index + 1] as usize],
        ))
    }
    /// Earcut-compatible polygon-local hole start indices (in vertex units).
    fn hole_indices(&self) -> impl Iterator<Item = u32> + '_ {
        let r = self.ring_offsets();
        let base = r.first().copied().unwrap_or(0);
        r.iter()
            .copied()
            .skip(1)
            .take(self.len().saturating_sub(1))
            .map(move |o| o - base)
    }
    fn area(&self) -> f64
    where
        T: Coord2d,
    {
        let mut area = self.exterior().ring_area();
        for interior in self.interiors() {
            area -= interior.ring_area();
        }
        area
    }
}

impl<T: Coord> PolygonRef<T> for Polygon<T> {
    fn coords(&self) -> &[T] {
        &self.coords
    }
    fn ring_offsets(&self) -> &[u32] {
        &self.ring_offsets
    }
}

impl<T: Coord> PolygonRef<T> for PolygonView<'_, T> {
    fn coords(&self) -> &[T] {
        self.coords
    }
    fn ring_offsets(&self) -> &[u32] {
        self.ring_offsets
    }
}

impl<T: Coord, R: PolygonRef<T> + ?Sized> PolygonRef<T> for &R {
    fn coords(&self) -> &[T] {
        (**self).coords()
    }
    fn ring_offsets(&self) -> &[u32] {
        (**self).ring_offsets()
    }
}

impl<'a, T: Coord> PolygonView<'a, T> {
    pub const fn from_raw(coords: &'a [T], ring_offsets: &'a [u32]) -> Self {
        Self {
            coords,
            ring_offsets,
        }
    }

    /// Materializes a self-contained owned `Polygon` (rebases ring_offsets to
    /// 0-based and copies just this polygon's coords) from this view.
    pub fn to_owned(&self) -> Polygon<T> {
        let (coords, ring_offsets) = rebase_offsets(self.coords, self.ring_offsets);
        Polygon::from_raw_unchecked(coords, ring_offsets)
    }
}

impl<T: Coord> Default for Polygon<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Coord> Polygon<T> {
    /// Creates an empty Polygon.
    pub fn new() -> Self {
        Self {
            coords: Vec::new(),
            ring_offsets: vec![0],
        }
    }

    pub fn from_raw(coords: Vec<T>, ring_offsets: Vec<u32>) -> Self {
        validate_offsets(coords.len(), &ring_offsets, "ring_offsets");
        Self {
            coords,
            ring_offsets,
        }
    }

    pub fn from_raw_unchecked(coords: Vec<T>, ring_offsets: Vec<u32>) -> Self {
        Self {
            coords,
            ring_offsets,
        }
    }

    /// Remove all rings from the polygon.
    pub fn clear(&mut self) {
        self.coords.clear();
        self.ring_offsets.clear();
        self.ring_offsets.push(0);
    }

    /// Adds an exterior or interior ring to the polygon.
    pub fn add_ring<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        extend_ring_dedup_closing(&mut self.coords, &mut self.ring_offsets, iter);
    }

    /// Create a new Polygon by applying the given transformation to all coordinates.
    pub fn transform<T2: Coord>(&self, f: impl Fn(&T) -> T2) -> Polygon<T2> {
        Polygon {
            coords: self.coords.iter().map(f).collect(),
            ring_offsets: self.ring_offsets.clone(),
        }
    }

    /// Applies the given transformation to all coordinates in the Polygon.
    pub fn transform_inplace(&mut self, mut f: impl FnMut(&T) -> T) {
        self.coords.iter_mut().for_each(|c| {
            *c = f(c);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_polygon_empty() {
        let polygon = Polygon2::<f64>::new();
        assert_eq!(polygon.exterior().len(), 0);
        assert_eq!(polygon.interiors().count(), 0);
    }

    #[test]
    fn test_polygon_no_interior() {
        let coords: Vec<_> = (0..=10).map(|i| [i as f64, i as f64]).collect();
        let ring_offsets: Vec<u32> = vec![0, 11];
        let polygon: Polygon2<f64> = Polygon2::from_raw_unchecked(coords, ring_offsets);

        assert_eq!(polygon.exterior().len(), 11);
        assert_eq!(polygon.interiors().count(), 0);
    }

    #[test]
    fn test_polygon_one_interior() {
        let coords: Vec<_> = (0..=10).map(|i| [i as f64, i as f64]).collect();
        let ring_offsets: Vec<u32> = vec![0, 4, 11];
        let polygon: Polygon2<f64> = Polygon2::from_raw_unchecked(coords, ring_offsets);

        assert_eq!(polygon.exterior().len(), 4);
        assert_eq!(polygon.interiors().count(), 1);
        for (i, interior) in polygon.interiors().enumerate() {
            match i {
                0 => assert_eq!(interior.len(), 7),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_polygon_multiple_interiors() {
        let coords: Vec<_> = (0..=10).map(|i| [i as f64, i as f64]).collect();
        let ring_offsets: Vec<u32> = vec![0, 4, 7, 11];
        let polygon: Polygon2<f64> = Polygon2::from_raw_unchecked(coords, ring_offsets);

        assert_eq!(polygon.exterior().len(), 4);
        assert_eq!(polygon.interiors().count(), 2);
        for (i, interior) in polygon.interiors().enumerate() {
            match i {
                0 => assert_eq!(interior.len(), 3),
                1 => assert_eq!(interior.len(), 4),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_polygon_clear() {
        let coords: Vec<_> = (0..=10).map(|i| [i as f64, i as f64]).collect();
        let ring_offsets: Vec<u32> = vec![0, 4, 7, 11];
        let mut polygon: Polygon2<f64> = Polygon2::from_raw_unchecked(coords, ring_offsets);

        assert_eq!(polygon.exterior().len(), 4);
        assert_eq!(polygon.interiors().count(), 2);

        polygon.clear();
        assert_eq!(polygon.exterior().len(), 0);
        assert_eq!(polygon.interiors().count(), 0);
    }

    #[test]
    fn test_polygon_add_ring() {
        let mut polygon = Polygon2::new();
        assert_eq!(polygon.exterior().len(), 0);
        assert_eq!(polygon.interiors().count(), 0);
        polygon.add_ring([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        assert_eq!(polygon.exterior().len(), 3);
        assert_eq!(polygon.interiors().count(), 0);
        polygon.add_ring([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        assert_eq!(polygon.exterior().len(), 3);
        assert_eq!(polygon.interiors().count(), 1);

        let mut polygon = Polygon2::new();
        polygon.add_ring([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]);
        assert_eq!(polygon.exterior().len(), 3);
        assert_eq!(polygon.interiors().count(), 0);
    }

    #[test]
    fn test_transform() {
        {
            let mut poly: Polygon2 = Polygon2::new();
            poly.add_ring([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);
            let new_poly = poly.transform(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                new_poly.exterior().raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }

        {
            let mut poly = Polygon2::new();
            poly.add_ring([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);
            poly.transform_inplace(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                poly.exterior().raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }
    }

    /// Currently, it does not check whether the exterior is valid (have at least three vertices) or not
    #[test]
    fn test_polygon_invalid_exterior() {
        let coords: Vec<_> = (0..1).map(|i| [i as f64, i as f64]).collect();
        let ring_offsets: Vec<u32> = vec![0, 1];
        let polygon: Polygon2<f64> = Polygon2::from_raw(coords, ring_offsets);

        assert_eq!(polygon.exterior().len(), 1); // only one vertex
        assert_eq!(polygon.interiors().count(), 0);
    }

    /// Currently, it does not check whether the interior is valid (have at least three vertices) or not
    /// (It could be a "Steiner point", as we see in earcut)
    #[test]
    fn test_polygon_invalid_interior() {
        let coords: Vec<_> = (0..=10).map(|i| [i as f64, i as f64]).collect();
        let ring_offsets: Vec<u32> = vec![0, 10, 11]; // only one vertex
        let polygon: Polygon2<f64> = Polygon2::from_raw(coords, ring_offsets);

        assert_eq!(polygon.exterior().len(), 10);
        assert_eq!(polygon.interiors().count(), 1);
        for (i, interior) in polygon.interiors().enumerate() {
            match i {
                0 => assert_eq!(interior.len(), 1), // only one vertex
                _ => unreachable!(),
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_polygon_invalid_ring_offsets_1() {
        let coords: Vec<_> = (0..=2).map(|i| [i as f64, i as f64]).collect();
        let ring_offsets: Vec<u32> = vec![0, 99]; // out of `coords` range
        let _polygon: Polygon2<f64> = Polygon2::from_raw(coords, ring_offsets);
    }

    #[test]
    #[should_panic]
    fn test_polygon_invalid_ring_offsets_2() {
        let coords: Vec<_> = (0..15).map(|i| [i as f64, i as f64]).collect();
        let ring_offsets: Vec<u32> = vec![0, 6, 3, 15]; // not monotonically increasing
        let _polygon: Polygon2<f64> = Polygon2::from_raw(coords, ring_offsets);
    }

    #[test]
    fn test_area() {
        let mut polygon = Polygon2::new();
        assert_eq!(polygon.area(), 0.0);
        polygon.add_ring([[0.0, 0.0], [3.0, 0.0], [3.0, 3.0], [0.0, 3.0]]);
        assert_eq!(polygon.area(), 9.0);
        polygon.add_ring([[1.0, 1.0], [1.0, 2.0], [2.0, 2.0], [2.0, 1.0]]);
        assert_eq!(polygon.area(), 8.0);

        // winding order should not matter
        let mut polygon = Polygon2::new();
        polygon.add_ring([[0.0, 0.0], [0.0, 3.0], [3.0, 3.0], [3.0, 0.0]]);
        assert_eq!(polygon.area(), 9.0);
        polygon.add_ring([[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0]]);
        assert_eq!(polygon.area(), 8.0);
    }
}
