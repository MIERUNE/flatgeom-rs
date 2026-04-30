//! Verifies that flatgeom geometries flow through `geo-traits`'
//! `to_geo` blanket helpers into `geo-types`.

#![cfg(feature = "geo-traits")]

use flatgeom::{LineString, MultiPolygon, Point, Polygon};
use geo_traits::to_geo::{ToGeoLineString, ToGeoMultiPolygon, ToGeoPoint, ToGeoPolygon};

#[test]
fn point_to_geo() {
    let p: Point<[f64; 2]> = Point::new([1.5, 2.5]);
    let gt: geo_types::Point<f64> = p.to_point();
    assert_eq!(gt.x(), 1.5);
    assert_eq!(gt.y(), 2.5);
}

#[test]
fn linestring_to_geo() {
    let ls: LineString<[f64; 2]> = LineString::from_raw(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
    let gt: geo_types::LineString<f64> = ls.to_line_string();
    assert_eq!(gt.0.len(), 3);
    assert_eq!(gt.0[1].x, 1.0);
    assert_eq!(gt.0[2].y, 0.0);
}

#[test]
fn polygon_with_hole_to_geo() {
    let mut p: Polygon<[f64; 2]> = Polygon::new();
    p.add_ring([[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]);
    p.add_ring([[2.0, 2.0], [4.0, 2.0], [4.0, 4.0], [2.0, 4.0]]);
    let gt: geo_types::Polygon<f64> = p.to_polygon();
    assert_eq!(gt.exterior().0.len(), 5); // closed
    assert_eq!(gt.interiors().len(), 1);
    assert_eq!(gt.interiors()[0].0.len(), 5);
}

#[test]
fn multipolygon_to_geo() {
    let mut mp: MultiPolygon<[f64; 2]> = MultiPolygon::new();
    mp.add_exterior([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
    mp.add_exterior([[10.0, 10.0], [11.0, 10.0], [11.0, 11.0]]);
    let gt: geo_types::MultiPolygon<f64> = mp.to_multi_polygon();
    assert_eq!(gt.0.len(), 2);
}

#[test]
fn integer_coords_to_geo() {
    // i32 coords still satisfy the tightened CoordNum bounds.
    let mut p: Polygon<[i32; 2]> = Polygon::new();
    p.add_ring([[0, 0], [10, 0], [10, 10], [0, 10]]);
    let gt: geo_types::Polygon<i32> = p.to_polygon();
    assert_eq!(gt.exterior().0.len(), 5);
}
