//! geo-traits implementations for flatgeom geometry types.
//!
//! Enabled via the `geo-traits` Cargo feature. Generic over coordinate
//! dimension `N` (any `usize`):
//!
//! - `N = 2`  → `Dimensions::Xy`
//! - `N = 3`  → `Dimensions::Xyz`
//! - `N = 4`  → `Dimensions::Xyzm`
//! - other N → `Dimensions::Unknown(N)` (1D, 5D, …; useful for index
//!   buffers, mesh topology, custom dimensions, etc.)
//!
//! Module layout mirrors `src/geometry/`.

use geo_traits::{CoordTrait, Dimensions};

use crate::CoordNum;

mod geometry;
mod line_string;
mod multi_line_string;
mod multi_point;
mod multi_polygon;
mod point;
mod polygon;

/// Compile-time map from array length `N` to a [`Dimensions`] tag.
///
/// `N` must be at least 2: 1-D geometries (e.g. `Polygon<[u32; 1]>` used as
/// index buffers / mesh topology) intentionally do not participate in
/// geo-traits, since they don't represent geographic coordinates. The
/// assertion fires at compile time when invoked from a `const` context
/// (see [`geom_trait_body!`]).
pub const fn dim_of<const N: usize>() -> Dimensions {
    assert!(
        N >= 2,
        "flatgeom geo-traits: dimension N must be >= 2 (N = 1 is reserved for index buffers)",
    );
    match N {
        2 => Dimensions::Xy,
        3 => Dimensions::Xyz,
        4 => Dimensions::Xyzm,
        _ => Dimensions::Unknown(N),
    }
}

// =====================================================================
// Wrappers
// =====================================================================

/// Borrowed reference to a fixed-size coordinate that implements [`CoordTrait`].
#[derive(Debug, Clone, Copy)]
pub struct CoordRef<'a, T, const N: usize>(pub &'a [T; N]);

impl<T: CoordNum, const N: usize> CoordTrait for CoordRef<'_, T, N> {
    type T = T;
    fn dim(&self) -> Dimensions {
        dim_of::<N>()
    }
    /// Panics if `N < 1`.
    fn x(&self) -> T {
        self.0[0]
    }
    /// Panics if `N < 2`.
    fn y(&self) -> T {
        self.0[1]
    }
    fn nth_or_panic(&self, n: usize) -> T {
        self.0[n]
    }
}

// =====================================================================
// Macros (private to this module tree)
// =====================================================================
//
// `geom_trait_body!` emits the 10 GeometryTrait associated types + dim()
// + as_type(). Only the matching variant is the "real" type; the others
// are Unimplemented*. `unimpl!` is a tt-muncher that picks "real" or
// `Unimplemented*<T>` per slot.

macro_rules! geom_trait_body {
    (
        $variant:ident, $real_kind:ident = $real:ty, dim = $n:ident
    ) => {
        type T = T;
        type PointType<'a>
            = $crate::geo_traits::unimpl!(Point, $real_kind, $real)
        where
            Self: 'a;
        type LineStringType<'a>
            = $crate::geo_traits::unimpl!(LineString, $real_kind, $real)
        where
            Self: 'a;
        type PolygonType<'a>
            = $crate::geo_traits::unimpl!(Polygon, $real_kind, $real)
        where
            Self: 'a;
        type MultiPointType<'a>
            = $crate::geo_traits::unimpl!(MultiPoint, $real_kind, $real)
        where
            Self: 'a;
        type MultiLineStringType<'a>
            = $crate::geo_traits::unimpl!(MultiLineString, $real_kind, $real)
        where
            Self: 'a;
        type MultiPolygonType<'a>
            = $crate::geo_traits::unimpl!(MultiPolygon, $real_kind, $real)
        where
            Self: 'a;
        type GeometryCollectionType<'a>
            = $crate::geo_traits::unimpl!(GeometryCollection, $real_kind, $real)
        where
            Self: 'a;
        type RectType<'a>
            = ::geo_traits::UnimplementedRect<T>
        where
            Self: 'a;
        type TriangleType<'a>
            = ::geo_traits::UnimplementedTriangle<T>
        where
            Self: 'a;
        type LineType<'a>
            = ::geo_traits::UnimplementedLine<T>
        where
            Self: 'a;

        fn dim(&self) -> ::geo_traits::Dimensions {
            const { $crate::geo_traits::dim_of::<$n>() }
        }
        fn as_type(
            &self,
        ) -> ::geo_traits::GeometryType<
            '_,
            Self::PointType<'_>,
            Self::LineStringType<'_>,
            Self::PolygonType<'_>,
            Self::MultiPointType<'_>,
            Self::MultiLineStringType<'_>,
            Self::MultiPolygonType<'_>,
            Self::GeometryCollectionType<'_>,
            Self::RectType<'_>,
            Self::TriangleType<'_>,
            Self::LineType<'_>,
        > {
            ::geo_traits::GeometryType::$variant(self)
        }
    };
}
pub(crate) use geom_trait_body;

macro_rules! unimpl {
    (Point,              Point,              $r:ty) => { $r };
    (LineString,         LineString,         $r:ty) => { $r };
    (Polygon,            Polygon,            $r:ty) => { $r };
    (MultiPoint,         MultiPoint,         $r:ty) => { $r };
    (MultiLineString,    MultiLineString,    $r:ty) => { $r };
    (MultiPolygon,       MultiPolygon,       $r:ty) => { $r };
    (GeometryCollection, GeometryCollection, $r:ty) => { $r };
    (Point,              $other:ident, $r:ty) => { ::geo_traits::UnimplementedPoint<T> };
    (LineString,         $other:ident, $r:ty) => { ::geo_traits::UnimplementedLineString<T> };
    (Polygon,            $other:ident, $r:ty) => { ::geo_traits::UnimplementedPolygon<T> };
    (MultiPoint,         $other:ident, $r:ty) => { ::geo_traits::UnimplementedMultiPoint<T> };
    (MultiLineString,    $other:ident, $r:ty) => { ::geo_traits::UnimplementedMultiLineString<T> };
    (MultiPolygon,       $other:ident, $r:ty) => { ::geo_traits::UnimplementedMultiPolygon<T> };
    (GeometryCollection, $other:ident, $r:ty) => { ::geo_traits::UnimplementedGeometryCollection<T> };
}
pub(crate) use unimpl;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_for_various_n() {
        assert_eq!(dim_of::<2>(), Dimensions::Xy);
        assert_eq!(dim_of::<3>(), Dimensions::Xyz);
        assert_eq!(dim_of::<4>(), Dimensions::Xyzm);
        assert_eq!(dim_of::<5>(), Dimensions::Unknown(5));
        assert_eq!(dim_of::<7>(), Dimensions::Unknown(7));
    }

    #[test]
    #[should_panic(expected = "N must be >= 2")]
    fn dim_of_1_panics() {
        // `dim_of::<1>()` panics; in `const` context this becomes a
        // compile error (see the `geom_trait_body!` macro which wraps
        // the call in `const { ... }`).
        let _ = dim_of::<1>();
    }

    #[test]
    fn coord_ref_basic() {
        let arr = [1.5_f64, 2.5];
        let c = CoordRef(&arr);
        assert_eq!(c.dim(), Dimensions::Xy);
        assert_eq!(c.x(), 1.5);
        assert_eq!(c.y(), 2.5);
    }
}
