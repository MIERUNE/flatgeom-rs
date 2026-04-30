//! Shared helpers for offset arrays used by jagged geometries
//! (`Polygon`, `MultiLineString`, `MultiPolygon`).

use alloc::vec::Vec;

/// Validates that `offsets` is a well-formed CSR-style sentinel array for a
/// values buffer of length `values_len`:
/// non-empty, starts with 0, monotonically non-decreasing, and ends at
/// exactly `values_len`. `name` is used in panic messages for context.
pub(super) fn validate_offsets(values_len: usize, offsets: &[u32], name: &str) {
    if offsets.is_empty() {
        panic!("{name} must contain at least the 0 sentinel");
    }
    if offsets[0] != 0 {
        panic!("{name} must start with 0");
    }
    if offsets.last().copied() != Some(values_len as u32) {
        panic!("{name} must end with values length");
    }
    if offsets.windows(2).any(|a| a[0] > a[1]) {
        panic!("{name} must be monotonically increasing");
    }
}

/// Extends `coords` with the contents of `iter`, drops the trailing point if
/// it duplicates the first point of the just-added ring, and pushes the new
/// end offset onto `offsets`. Used by `Polygon::add_ring` and
/// `MultiPolygon::add_ring`.
pub(super) fn extend_ring_dedup_closing<T, I>(coords: &mut Vec<T>, offsets: &mut Vec<u32>, iter: I)
where
    T: PartialEq,
    I: IntoIterator<Item = T>,
{
    let head = coords.len();
    coords.extend(iter);
    let tail = coords.len();
    if tail > head + 2 && coords[head..head + 1] == coords[tail - 1..] {
        coords.truncate(tail - 1);
    }
    offsets.push(coords.len() as u32);
}

/// Materializes a self-contained `(coords, offsets)` pair from a view's
/// possibly-absolute `offsets` over a parent buffer: copies just the slice
/// covered by `offsets`, and rebases the offsets to start at 0. Used by
/// `PolygonView::to_owned` and `MultiLineStringView::to_owned`.
pub(super) fn rebase_offsets<T: Copy>(coords: &[T], offsets: &[u32]) -> (Vec<T>, Vec<u32>) {
    let base = offsets.first().copied().unwrap_or(0);
    let last = offsets.last().copied().unwrap_or(0);
    let owned_coords = coords[base as usize..last as usize].to_vec();
    let owned_offsets: Vec<u32> = offsets.iter().map(|o| o - base).collect();
    (owned_coords, owned_offsets)
}

// =====================================================================
// serde helpers
// =====================================================================

/// Wraps an iterator of `u32` lengths so that it serializes as a flat
/// sequence of length values (without materializing a `Vec`).
#[cfg(feature = "serde")]
pub(super) struct OffsetLengths<I>(pub I);

#[cfg(feature = "serde")]
impl<I> serde::Serialize for OffsetLengths<I>
where
    I: Clone + ExactSizeIterator<Item = u32>,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;

        let iter = self.0.clone();
        let mut seq = serializer.serialize_seq(Some(iter.len()))?;
        for len in iter {
            seq.serialize_element(&len)?;
        }
        seq.end()
    }
}

/// Reconstructs CSR-style offsets (`[0, l0, l0 + l1, ...]`) from a length
/// sequence, validating that the running total matches `values_len`.
#[cfg(feature = "serde")]
pub(super) fn offsets_from_lengths<E: serde::de::Error>(
    lengths: &[u32],
    values_len: usize,
    name: &str,
) -> Result<Vec<u32>, E> {
    let mut offsets = Vec::with_capacity(lengths.len() + 1);
    let mut acc = 0_u32;
    offsets.push(acc);
    for &len in lengths {
        acc = acc
            .checked_add(len)
            .ok_or_else(|| E::custom(format_args!("{name} overflow")))?;
        offsets.push(acc);
    }
    if acc as usize != values_len {
        return Err(E::custom(format_args!(
            "{name} total length {} does not match values length {}",
            acc, values_len
        )));
    }
    Ok(offsets)
}
