use crate::{
    Coord, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};

/// Computer-friendly Geometry
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone)]
pub enum Geometry<T: Coord> {
    Point(Point<T>),
    MultiPoint(MultiPoint<T>),
    LineString(LineString<T>),
    MultiLineString(MultiLineString<T>),
    Polygon(Polygon<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
}

pub type Geometry2<C = f64> = Geometry<[C; 2]>;
pub type Geometry3<C = f64> = Geometry<[C; 3]>;
