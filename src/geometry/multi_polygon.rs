use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;

use super::{
    Coord,
    offsets::{extend_ring_dedup_closing, validate_offsets},
    polygon::{Polygon, PolygonRef, PolygonView},
};

#[cfg(feature = "serde")]
use super::offsets::offsets_from_lengths;

/// Computer-friendly MultiPolygon
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPolygon<T: Coord> {
    /// All coordinates of all polygons
    ///
    /// e.g. `[x0, y0, z0, x1, y1, z1, ...]`
    coords: Vec<T>,

    /// A sequence of offsets into coords from which each ring starts.
    ///
    /// The first offset is always 0, and the last offset is always coords.len().
    ring_offsets: Vec<u32>,

    /// A sequence of offsets into ring_offsets from which each polygon starts.
    ///
    /// The first offset is always 0, and the last offset is always the ring count.
    polygon_offsets: Vec<u32>,
}

pub type MultiPolygon2<C = f64> = MultiPolygon<[C; 2]>;
pub type MultiPolygon3<C = f64> = MultiPolygon<[C; 3]>;

/// Read-only borrowed view of a MultiPolygon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultiPolygonView<'a, T: Coord> {
    coords: &'a [T],
    ring_offsets: &'a [u32],
    polygon_offsets: &'a [u32],
}

pub type MultiPolygonView2<'a, C = f64> = MultiPolygonView<'a, [C; 2]>;
pub type MultiPolygonView3<'a, C = f64> = MultiPolygonView<'a, [C; 3]>;

/// Read-only abstraction over [`MultiPolygon`] and [`MultiPolygonView`].
pub trait MultiPolygonRef<T: Coord> {
    fn coords(&self) -> &[T];
    fn ring_offsets(&self) -> &[u32];
    fn polygon_offsets(&self) -> &[u32];

    fn raw_coords(&self) -> &[T] {
        let polygon_offsets = self.polygon_offsets();
        let ring_offsets = self.ring_offsets();
        let Some(&polygon_start) = polygon_offsets.first() else {
            return &[];
        };
        let Some(&polygon_end) = polygon_offsets.last() else {
            return &[];
        };
        let start = ring_offsets
            .get(polygon_start as usize)
            .copied()
            .unwrap_or(0);
        let end = ring_offsets.get(polygon_end as usize).copied().unwrap_or(0);
        &self.coords()[start as usize..end as usize]
    }
    /// Returns per-polygon ring counts, derived as deltas of consecutive polygon offsets.
    fn polygon_ring_counts(&self) -> impl Clone + ExactSizeIterator<Item = u32> + '_ {
        self.polygon_offsets()
            .windows(2)
            .map(|window| window[1] - window[0])
    }
    /// Returns ring vertex counts for this multipolygon, derived as deltas of ring offsets.
    fn ring_lengths(&self) -> impl Clone + ExactSizeIterator<Item = u32> + '_ {
        let empty = &[];
        let polygon_offsets = self.polygon_offsets();
        let ring_offsets = self.ring_offsets();
        let offsets = match (polygon_offsets.first(), polygon_offsets.last()) {
            (Some(&start), Some(&end)) => ring_offsets
                .get(start as usize..=end as usize)
                .unwrap_or(empty),
            _ => empty,
        };
        offsets.windows(2).map(|window| window[1] - window[0])
    }
    fn len(&self) -> usize {
        self.polygon_offsets().len().saturating_sub(1)
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn iter<'a>(&'a self) -> Iter<'a, T>
    where
        T: 'a,
    {
        let r_prev = self.polygon_offsets().first().copied().unwrap_or(0);
        Iter {
            coords: self.coords(),
            ring_offsets: self.ring_offsets(),
            polygon_offsets: self.polygon_offsets(),
            pos: 0,
            end: self.len(),
            r_prev,
        }
    }
    fn iter_range<'a>(&'a self, range: Range<usize>) -> Iter<'a, T>
    where
        T: 'a,
    {
        let r_prev = self
            .polygon_offsets()
            .get(range.start)
            .copied()
            .unwrap_or(0);
        Iter {
            coords: self.coords(),
            ring_offsets: self.ring_offsets(),
            polygon_offsets: self.polygon_offsets(),
            pos: range.start,
            end: range.end,
            r_prev,
        }
    }
    fn get(&self, index: usize) -> PolygonView<'_, T> {
        let len = self.len();
        if index >= len {
            panic!(
                "index out of bounds: {} polygon(s) but index is {}",
                len, index
            );
        }
        let polygon_offsets = self.polygon_offsets();
        let r_start = polygon_offsets[index] as usize;
        let r_end = polygon_offsets[index + 1] as usize;
        PolygonView::from_raw(self.coords(), &self.ring_offsets()[r_start..=r_end])
    }
}

impl<T: Coord> MultiPolygonRef<T> for MultiPolygon<T> {
    fn coords(&self) -> &[T] {
        &self.coords
    }
    fn ring_offsets(&self) -> &[u32] {
        &self.ring_offsets
    }
    fn polygon_offsets(&self) -> &[u32] {
        &self.polygon_offsets
    }
}

impl<T: Coord> MultiPolygonRef<T> for MultiPolygonView<'_, T> {
    fn coords(&self) -> &[T] {
        self.coords
    }
    fn ring_offsets(&self) -> &[u32] {
        self.ring_offsets
    }
    fn polygon_offsets(&self) -> &[u32] {
        self.polygon_offsets
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for MultiPolygon<T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_multi_polygon(self, serializer)
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for MultiPolygonView<'_, T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_multi_polygon(self, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for MultiPolygon<T>
where
    T: Coord + serde::Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct MultiPolygonSerde<T> {
            coords: Vec<T>,
            topology: Vec<u32>,
        }

        let value = MultiPolygonSerde::deserialize(deserializer)?;
        let (ring_offsets, polygon_offsets) =
            offsets_from_topology::<D::Error>(&value.topology, value.coords.len())?;
        Ok(MultiPolygon::from_raw_unchecked(
            value.coords,
            ring_offsets,
            polygon_offsets,
        ))
    }
}

#[cfg(feature = "serde")]
fn serialize_multi_polygon<T, M, S>(multi_polygon: &M, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Coord + serde::Serialize,
    M: MultiPolygonRef<T> + ?Sized,
    S: serde::Serializer,
{
    use serde::ser::SerializeStruct;

    let mut s = serializer.serialize_struct("MultiPolygon", 2)?;
    s.serialize_field("coords", multi_polygon.raw_coords())?;
    s.serialize_field(
        "topology",
        &MultiPolygonTopology {
            polygon_count: multi_polygon.len(),
            polygon_ring_counts: multi_polygon.polygon_ring_counts(),
            ring_lengths: multi_polygon.ring_lengths(),
        },
    )?;
    s.end()
}

#[cfg(feature = "serde")]
struct MultiPolygonTopology<P, R> {
    polygon_count: usize,
    polygon_ring_counts: P,
    ring_lengths: R,
}

#[cfg(feature = "serde")]
impl<P, R> serde::Serialize for MultiPolygonTopology<P, R>
where
    P: Clone + ExactSizeIterator<Item = u32>,
    R: Clone + ExactSizeIterator<Item = u32>,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;

        let polygon_ring_counts = self.polygon_ring_counts.clone();
        let ring_lengths = self.ring_lengths.clone();
        let mut seq =
            serializer.serialize_seq(Some(1 + polygon_ring_counts.len() + ring_lengths.len()))?;

        seq.serialize_element(&(self.polygon_count as u32))?;
        for count in polygon_ring_counts {
            seq.serialize_element(&count)?;
        }
        for len in ring_lengths {
            seq.serialize_element(&len)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
fn offsets_from_topology<E: serde::de::Error>(
    topology: &[u32],
    coord_len: usize,
) -> Result<(Vec<u32>, Vec<u32>), E> {
    if topology.is_empty() {
        return Err(E::custom("topology must contain polygon count"));
    }

    let polygon_count = topology[0] as usize;
    if topology.len() < 1 + polygon_count {
        return Err(E::custom("topology is shorter than polygon ring counts"));
    }

    let ring_counts = &topology[1..1 + polygon_count];
    let total_rings = checked_sum::<E>(ring_counts, "polygon ring counts")? as usize;
    if topology.len() != 1 + polygon_count + total_rings {
        return Err(E::custom("topology length does not match ring counts"));
    }

    let ring_lengths = &topology[1 + polygon_count..];
    let ring_offsets = offsets_from_lengths::<E>(ring_lengths, coord_len, "ring lengths")?;
    let polygon_offsets =
        offsets_from_lengths::<E>(ring_counts, total_rings, "polygon ring counts")?;
    Ok((ring_offsets, polygon_offsets))
}

#[cfg(feature = "serde")]
fn checked_sum<E: serde::de::Error>(values: &[u32], name: &str) -> Result<u32, E> {
    let mut acc = 0_u32;
    for &value in values {
        acc = acc
            .checked_add(value)
            .ok_or_else(|| E::custom(format_args!("{name} overflow")))?;
    }
    Ok(acc)
}

impl<'a, T: Coord> MultiPolygonView<'a, T> {
    pub const fn from_raw(
        coords: &'a [T],
        ring_offsets: &'a [u32],
        polygon_offsets: &'a [u32],
    ) -> Self {
        Self {
            coords,
            ring_offsets,
            polygon_offsets,
        }
    }

    /// Materializes a self-contained owned `MultiPolygon` (rebases both
    /// `polygon_offsets` and `ring_offsets` to 0-based and copies just this
    /// multi's coords).
    pub fn to_owned(&self) -> MultiPolygon<T> {
        let po = self.polygon_offsets;
        let ro = self.ring_offsets;
        if po.len() < 2 {
            return MultiPolygon::new();
        }
        let p_base = po.first().copied().unwrap_or(0);
        let p_last = po.last().copied().unwrap_or(0);
        let r_start = ro.get(p_base as usize).copied().unwrap_or(0);
        let r_end = ro.get(p_last as usize).copied().unwrap_or(0);
        let coords = self.coords[r_start as usize..r_end as usize].to_vec();
        let ring_offsets: Vec<u32> = ro[p_base as usize..=p_last as usize]
            .iter()
            .map(|o| o - r_start)
            .collect();
        let polygon_offsets: Vec<u32> = po.iter().map(|o| o - p_base).collect();
        MultiPolygon::from_raw_unchecked(coords, ring_offsets, polygon_offsets)
    }
}

impl<T: Coord> Default for MultiPolygon<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Coord> MultiPolygon<T> {
    /// Creates an empty MultiPolygon.
    pub fn new() -> Self {
        Self {
            coords: Vec::new(),
            ring_offsets: vec![0],
            polygon_offsets: vec![0],
        }
    }

    /// Create a new multipolygon from the given raw data, checking for validity.
    pub fn from_raw(coords: Vec<T>, ring_offsets: Vec<u32>, polygon_offsets: Vec<u32>) -> Self {
        validate_offsets(coords.len(), &ring_offsets, "ring_offsets");
        validate_offsets(ring_offsets.len() - 1, &polygon_offsets, "polygon_offsets");

        Self {
            coords,
            ring_offsets,
            polygon_offsets,
        }
    }

    /// Creates a new multipolygon from the given raw data, without validity check.
    pub fn from_raw_unchecked(
        coords: Vec<T>,
        ring_offsets: Vec<u32>,
        polygon_offsets: Vec<u32>,
    ) -> Self {
        Self {
            coords,
            ring_offsets,
            polygon_offsets,
        }
    }

    /// Clears the multipolygon, removing all polygons.
    pub fn clear(&mut self) {
        self.coords.clear();
        self.ring_offsets.clear();
        self.ring_offsets.push(0);
        self.polygon_offsets.clear();
        self.polygon_offsets.push(0);
    }

    /// Adds a polygon to the multipolygon.
    pub fn push(&mut self, poly: &Polygon<T>) {
        self.add_exterior(&poly.exterior());
        for hole in poly.interiors() {
            self.add_interior(&hole);
        }
    }

    /// Adds a polygon with the given exterior ring.
    pub fn add_exterior<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.add_ring(iter);
        self.polygon_offsets
            .push((self.ring_offsets.len() - 1) as u32);
    }

    /// Adds an interior ring to the last polygon.
    pub fn add_interior<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        if self.is_empty() {
            self.add_ring(core::iter::empty());
        } else {
            self.polygon_offsets.pop();
        }
        self.add_ring(iter);
        self.polygon_offsets
            .push((self.ring_offsets.len() - 1) as u32);
    }

    /// Create a new MultiPolygon by applying the given transformation to all coordinates.
    pub fn transform<T2: Coord>(&self, f: impl Fn(&T) -> T2) -> MultiPolygon<T2> {
        MultiPolygon {
            coords: self.coords.iter().map(f).collect(),
            ring_offsets: self.ring_offsets.clone(),
            polygon_offsets: self.polygon_offsets.clone(),
        }
    }

    /// Applies the given transformation to all coordinates in the MultiPolygon.
    pub fn transform_inplace(&mut self, mut f: impl FnMut(&T) -> T) {
        self.coords.iter_mut().for_each(|c| {
            *c = f(c);
        });
    }

    fn add_ring<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        extend_ring_dedup_closing(&mut self.coords, &mut self.ring_offsets, iter);
    }
}

impl<'a, T: Coord> IntoIterator for &'a MultiPolygon<T> {
    type Item = PolygonView<'a, T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        <MultiPolygon<T> as MultiPolygonRef<T>>::iter(self)
    }
}

impl<'a, T: Coord> IntoIterator for &MultiPolygonView<'a, T> {
    type Item = PolygonView<'a, T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let r_prev = self.polygon_offsets.first().copied().unwrap_or(0);
        Iter {
            coords: self.coords,
            ring_offsets: self.ring_offsets,
            polygon_offsets: self.polygon_offsets,
            pos: 0,
            end: self.polygon_offsets.len().saturating_sub(1),
            r_prev,
        }
    }
}

pub struct Iter<'a, T: Coord> {
    coords: &'a [T],
    ring_offsets: &'a [u32],
    polygon_offsets: &'a [u32],
    pos: usize,
    end: usize,
    r_prev: u32,
}

impl<'a, T: Coord> Iterator for Iter<'a, T> {
    type Item = PolygonView<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.end {
            return None;
        }
        let r_next = self.polygon_offsets[self.pos + 1];
        let view = PolygonView::from_raw(
            self.coords,
            &self.ring_offsets[self.r_prev as usize..=r_next as usize],
        );
        self.r_prev = r_next;
        self.pos += 1;
        Some(view)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.end.saturating_sub(self.pos);
        (n, Some(n))
    }
}

impl<'a, T: Coord> ExactSizeIterator for Iter<'a, T> {}

#[cfg(test)]
mod tests {
    use super::{super::polygon::Polygon2, *};
    use crate::geometry::linestring::LineStringRef;
    use alloc::vec::Vec;

    #[test]
    fn test_mpoly_add() {
        let mut mpoly = MultiPolygon2::new();
        assert_eq!(mpoly.len(), 0);
        assert!(mpoly.is_empty());
        assert_eq!(mpoly.iter().count(), 0);

        // 1st polygon
        let mut poly1 = Polygon2::new();
        poly1.add_ring([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]); // exterior
        poly1.add_ring([[1., 1.], [2., 1.], [2., 2.], [1., 2.]]); // interior
        poly1.add_ring([[3., 3.], [4., 3.], [4., 4.], [3., 4.]]); // interior
        mpoly.push(&poly1);
        assert!(!mpoly.is_empty());
        assert_eq!(mpoly.len(), 1);

        // 2nd polygon
        let mut poly2 = Polygon2::new();
        poly2.add_ring([[4., 0.], [7., 0.], [7., 3.], [4., 3.]]); // exterior
        poly2.add_ring([[5., 1.], [6., 1.], [6., 2.], [5., 2.]]); // interior
        mpoly.push(&poly2);
        assert_eq!(mpoly.len(), 2);

        // 3rd polygon
        let mut poly3 = Polygon2::new();
        poly3.add_ring([[4., 0.], [7., 0.], [7., 3.], [4., 3.]]); // exterior
        mpoly.push(&poly3);
        assert_eq!(mpoly.len(), 3);

        for (i, poly) in mpoly.iter().enumerate() {
            match i {
                0 => {
                    assert_eq!(
                        poly.exterior().iter().collect::<Vec<_>>(),
                        [[0., 0.], [5., 0.], [5., 5.], [0., 5.]]
                    );
                    assert_eq!(poly.interiors().count(), 2);
                    assert_eq!(
                        poly.interiors()
                            .map(|r| r.iter().collect::<Vec<_>>())
                            .collect::<Vec<_>>(),
                        [
                            [[1., 1.], [2., 1.], [2., 2.], [1., 2.]],
                            [[3., 3.], [4., 3.], [4., 4.], [3., 4.]],
                        ]
                    );
                }
                1 => {
                    assert_eq!(
                        poly.exterior().iter().collect::<Vec<_>>(),
                        [[4., 0.], [7., 0.], [7., 3.], [4., 3.]]
                    );
                    assert_eq!(poly.interiors().count(), 1);
                    assert_eq!(
                        poly.interiors()
                            .map(|r| r.iter().collect::<Vec<_>>())
                            .collect::<Vec<_>>(),
                        [[[5., 1.], [6., 1.], [6., 2.], [5., 2.]]]
                    );
                }
                2 => {
                    assert_eq!(
                        poly.exterior().iter().collect::<Vec<_>>(),
                        [[4., 0.], [7., 0.], [7., 3.], [4., 3.]]
                    );
                    assert_eq!(poly.interiors().count(), 0);
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_mpoly_append() {
        let mut mpoly = MultiPolygon2::new();
        assert_eq!(mpoly.len(), 0);
        assert!(mpoly.is_empty());
        assert_eq!(mpoly.iter().count(), 0);

        // 1st polygon
        mpoly.add_exterior([[0., 0.], [5., 0.], [5., 5.], [0., 5.], [0., 0.]]);
        assert_eq!(mpoly.len(), 1);
        mpoly.add_interior([[1., 1.], [2., 1.], [2., 2.], [1., 2.], [1., 1.]]);
        assert_eq!(mpoly.len(), 1);
        mpoly.add_interior([[3., 3.], [4., 3.], [4., 4.], [3., 4.], [3., 3.]]);
        assert_eq!(mpoly.len(), 1);
        assert!(!mpoly.is_empty());
        for (i, poly) in mpoly.iter().enumerate() {
            match i {
                0 => assert_eq!(poly.exterior().len(), 4),
                _ => unreachable!(),
            }
        }

        // 2nd polygon
        mpoly.add_exterior([[4., 0.], [7., 0.], [7., 3.], [4., 3.], [4., 0.]]);
        assert_eq!(mpoly.len(), 2);
        mpoly.add_interior([[5., 1.], [6., 1.], [6., 2.], [5., 2.], [5., 1.]]);
        assert_eq!(mpoly.len(), 2);

        // 3rd polygon
        mpoly.add_exterior([[4., 0.], [7., 0.], [7., 3.], [4., 3.], [4., 0.]]);
        assert_eq!(mpoly.len(), 3);

        for (i, poly) in mpoly.iter().enumerate() {
            match i {
                0 => assert_eq!(poly.interiors().count(), 2),
                1 => assert_eq!(poly.interiors().count(), 1),
                2 => assert_eq!(poly.interiors().count(), 0),
                _ => unreachable!(),
            }
        }

        for (i, poly) in mpoly.iter_range(0..1).enumerate() {
            match i {
                0 => assert_eq!(poly.interiors().count(), 2),
                _ => unreachable!(),
            }
        }

        for (i, poly) in mpoly.iter_range(1..2).enumerate() {
            match i {
                0 => assert_eq!(poly.interiors().count(), 1),
                _ => unreachable!(),
            }
        }

        let mut found = false;
        for (i, poly) in mpoly.iter_range(2..3).enumerate() {
            match i {
                0 => {
                    assert_eq!(poly.interiors().count(), 0);
                    found = true;
                }
                _ => unreachable!(),
            }
        }
        assert!(found);

        mpoly.clear();
        assert_eq!(mpoly.len(), 0);
        assert!(mpoly.is_empty());
    }

    #[test]
    fn test_mpoly_empty_exterior() {
        let mut mpoly = MultiPolygon2::new();

        // Start the 1st polygon with an empty exterior
        mpoly.add_interior([[5., 1.], [6., 1.], [6., 2.], [5., 2.], [5., 1.]]);
        assert_eq!(mpoly.get(0).exterior().len(), 0);
        assert_eq!(mpoly.len(), 1);
        // Start the 2nd polygon
        mpoly.add_exterior([[4., 0.], [7., 0.], [7., 3.], [4., 3.], [4., 0.]]);
        assert_eq!(mpoly.len(), 2);
    }

    #[test]
    fn test_from_raw_valid() {
        // checked
        let _mpoly = MultiPolygon2::from_raw(
            [
                [0.0, 0.0],
                [5.0, 0.0],
                [5.0, 5.0],
                [0.0, 5.0],
                [1.0, 1.0],
                [2.0, 1.0],
                [2.0, 2.0],
                [1.0, 2.0],
                [3.0, 3.0],
                [4.0, 3.0],
                [4.0, 4.0],
                [3.0, 4.0],
                [4.0, 0.0],
                [7.0, 0.0],
                [7.0, 3.0],
                [4.0, 3.0],
                [5.0, 1.0],
                [6.0, 1.0],
                [6.0, 2.0],
                [5.0, 2.0],
                [4.0, 0.0],
                [7.0, 0.0],
                [7.0, 3.0],
                [4.0, 3.0],
                [5.0, 1.0],
                [6.0, 1.0],
                [6.0, 2.0],
                [5.0, 2.0],
            ]
            .to_vec(),
            [0, 4, 8, 12, 16, 20, 24, 28].to_vec(),
            [0, 3, 5, 7].to_vec(),
        );

        // unchecked
        let _mpoly = MultiPolygon2::from_raw_unchecked(
            [
                [0.0, 0.0],
                [5.0, 0.0],
                [5.0, 5.0],
                [0.0, 5.0],
                [1.0, 1.0],
                [2.0, 1.0],
                [2.0, 2.0],
                [1.0, 2.0],
                [3.0, 3.0],
                [4.0, 3.0],
                [4.0, 4.0],
                [3.0, 4.0],
                [4.0, 0.0],
                [7.0, 0.0],
                [7.0, 3.0],
                [4.0, 3.0],
                [5.0, 1.0],
                [6.0, 1.0],
                [6.0, 2.0],
                [5.0, 2.0],
                [4.0, 0.0],
                [7.0, 0.0],
                [7.0, 3.0],
                [4.0, 3.0],
                [5.0, 1.0],
                [6.0, 1.0],
                [6.0, 2.0],
                [5.0, 2.0],
            ]
            .to_vec(),
            [0, 4, 8, 12, 16, 20, 24, 28].to_vec(),
            [0, 3, 5, 7].to_vec(),
        );
    }

    #[test]
    fn test_transform() {
        {
            let mut mpoly = MultiPolygon2::new();
            mpoly.add_exterior([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);
            let new_mpoly = mpoly.transform(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                new_mpoly.get(0).raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }

        {
            let mut mpoly = MultiPolygon2::new();
            mpoly.add_exterior([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);
            mpoly.transform_inplace(|[x, y]| [x + 2., y + 1.]);
            assert_eq!(
                mpoly.get(0).raw_coords(),
                [[2., 1.], [7., 1.], [7., 6.], [2., 6.]]
            );
        }
    }

    #[test]
    #[should_panic]
    fn test_from_raw_invalid_1() {
        let _mpoly = MultiPolygon2::from_raw(
            [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 1.0]].to_vec(),
            [0, 1, 99].to_vec(), // invalid
            [0, 2].to_vec(),
        );
    }

    #[test]
    #[should_panic]
    fn test_from_raw_invalid_2() {
        let _mpoly = MultiPolygon2::from_raw(
            [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 1.0]].to_vec(),
            [0, 4].to_vec(),
            [0, 99].to_vec(), // invalid
        );
    }

    #[test]
    #[should_panic]
    fn test_from_raw_invalid_3() {
        let _mpoly = MultiPolygon2::from_raw(
            [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 1.0]].to_vec(),
            [1, 4].to_vec(), // invalid
            [0, 1].to_vec(),
        );
    }

    #[test]
    #[should_panic]
    fn test_from_raw_invalid_4() {
        // checked
        let _mpoly = MultiPolygon2::from_raw(
            [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 1.0]].to_vec(),
            [].to_vec(), // missing sentinel
            [0].to_vec(),
        );
    }

    #[test]
    #[should_panic]
    fn test_from_raw_invalid_5() {
        // checked
        let _mpoly = MultiPolygon2::from_raw(
            [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 1.0]].to_vec(),
            [0, 3, 2, 4].to_vec(), // not increasing
            [0, 3].to_vec(),
        );
    }

    #[test]
    #[should_panic]
    fn test_from_raw_invalid_6() {
        // checked
        let _mpoly = MultiPolygon2::from_raw(
            [[0., 0.], [1., 1.], [2., 2.], [3., 3.], [4., 4.], [5., 5.]].to_vec(),
            [0, 2, 4, 6].to_vec(),
            [0, 3, 1].to_vec(), // not increasing
        );
    }
}
