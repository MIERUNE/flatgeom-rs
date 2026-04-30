#[allow(clippy::module_inception)]
mod geometry;
mod geometry_collection;
mod linestring;
mod multi_linestring;
mod multi_point;
mod multi_polygon;
mod offsets;
mod point;
mod polygon;

pub use geometry::{Geometry, Geometry2, Geometry3};
pub use geometry_collection::{
    GeometryCollection, GeometryCollection2, GeometryCollection3, GeometryCollectionRef,
};
pub use linestring::{
    LineString, LineString2, LineString3, LineStringRef, LineStringView, LineStringView2,
    LineStringView3,
};
pub use multi_linestring::{
    MultiLineString, MultiLineString2, MultiLineString3, MultiLineStringRef, MultiLineStringView,
    MultiLineStringView2, MultiLineStringView3,
};
pub use multi_point::{
    MultiPoint, MultiPoint2, MultiPoint3, MultiPointRef, MultiPointView, MultiPointView2,
    MultiPointView3,
};
pub use multi_polygon::{
    MultiPolygon, MultiPolygon2, MultiPolygon3, MultiPolygonRef, MultiPolygonView,
    MultiPolygonView2, MultiPolygonView3,
};
pub use point::{Point, Point2, Point3, PointRef, PointView, PointView2, PointView3};
pub use polygon::{
    Polygon, Polygon2, Polygon3, PolygonRef, PolygonView, PolygonView2, PolygonView3,
};

use core::fmt::Debug;

use num_traits::{Num, NumCast};

pub trait Coord: Copy + PartialEq {}
/// Numeric type usable as a coordinate component.
///
/// Bounds are aligned with [`geo_types::CoordNum`] (`Num + Copy + NumCast +
/// PartialOrd + Debug`) so that flatgeom geometries integrate transparently
/// with `geo-traits` ↔ `geo-types` conversion helpers.
pub trait CoordNum: Num + Copy + NumCast + PartialOrd + Debug {}

impl<const D: usize, N: CoordNum> Coord for [N; D] {}
impl<N: CoordNum> Coord for N {}
impl CoordNum for f32 {}
impl CoordNum for f64 {}
impl CoordNum for u8 {}
impl CoordNum for i8 {}
impl CoordNum for u16 {}
impl CoordNum for i16 {}
impl CoordNum for u32 {}
impl CoordNum for i32 {}
impl CoordNum for u64 {}
impl CoordNum for i64 {}

pub trait Coord2d: Coord {
    fn xy(&self) -> (f64, f64);
}

impl<N: CoordNum> Coord2d for [N; 2] {
    fn xy(&self) -> (f64, f64) {
        (self[0].to_f64().unwrap(), self[1].to_f64().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_coord_num_trait() {
        // 2D LineString with floating point numbers
        let a: LineString2<f32> = LineString::from_raw(vec![[1.2, 2.3], [3.4, 4.5]]);
        assert_eq!(a.len(), 2);
        // Can also be used to store integer values (e.g., vertex indices)
        let b: LineString<u32> = LineString::from_raw(vec![1, 2, 3]);
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn coord2d() {
        let v = [1, 2];
        assert_eq!(v.xy(), (1.0, 2.0));

        let a: LineString2<f32> = LineString::from_raw(vec![[0.0, 0.0], [1.0, 1.0], [-2.0, 3.0]]);
        assert_eq!(a.ring_area(), 2.5);
    }
}
