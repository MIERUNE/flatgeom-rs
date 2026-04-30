use geo_traits::{
    Dimensions, GeometryCollectionTrait, GeometryTrait, GeometryType, UnimplementedLine,
    UnimplementedRect, UnimplementedTriangle,
};

use crate::geo_traits::{dim_of, geom_trait_body};
use crate::{
    CoordNum, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon,
};

// ----- GeometryCollection -----

impl<T: CoordNum, const N: usize> GeometryTrait for GeometryCollection<[T; N]> {
    geom_trait_body!(
        GeometryCollection,
        GeometryCollection = GeometryCollection<[T; N]>,
        dim = N
    );
}

impl<T: CoordNum, const N: usize> GeometryCollectionTrait for GeometryCollection<[T; N]> {
    type GeometryType<'a>
        = &'a Geometry<[T; N]>
    where
        Self: 'a;
    fn num_geometries(&self) -> usize {
        self.0.len()
    }
    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        unsafe { self.0.get_unchecked(i) }
    }
}

impl<'g, T: CoordNum + 'g, const N: usize> GeometryTrait for &'g GeometryCollection<[T; N]> {
    geom_trait_body!(
        GeometryCollection,
        GeometryCollection = GeometryCollection<[T; N]>,
        dim = N
    );
}

impl<'g, T: CoordNum + 'g, const N: usize> GeometryCollectionTrait
    for &'g GeometryCollection<[T; N]>
{
    type GeometryType<'a>
        = &'a Geometry<[T; N]>
    where
        Self: 'a;
    fn num_geometries(&self) -> usize {
        self.0.len()
    }
    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        unsafe { self.0.get_unchecked(i) }
    }
}

// ----- Geometry / &Geometry (full enum dispatch) -----

macro_rules! geometry_enum_trait_body {
    () => {
        type T = T;
        type PointType<'a>
            = Point<[T; N]>
        where
            Self: 'a;
        type LineStringType<'a>
            = LineString<[T; N]>
        where
            Self: 'a;
        type PolygonType<'a>
            = Polygon<[T; N]>
        where
            Self: 'a;
        type MultiPointType<'a>
            = MultiPoint<[T; N]>
        where
            Self: 'a;
        type MultiLineStringType<'a>
            = MultiLineString<[T; N]>
        where
            Self: 'a;
        type MultiPolygonType<'a>
            = MultiPolygon<[T; N]>
        where
            Self: 'a;
        type GeometryCollectionType<'a>
            = GeometryCollection<[T; N]>
        where
            Self: 'a;
        type RectType<'a>
            = UnimplementedRect<T>
        where
            Self: 'a;
        type TriangleType<'a>
            = UnimplementedTriangle<T>
        where
            Self: 'a;
        type LineType<'a>
            = UnimplementedLine<T>
        where
            Self: 'a;

        fn dim(&self) -> Dimensions {
            const { dim_of::<N>() }
        }
        fn as_type(
            &self,
        ) -> GeometryType<
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
            match self {
                Geometry::Point(g) => GeometryType::Point(g),
                Geometry::MultiPoint(g) => GeometryType::MultiPoint(g),
                Geometry::LineString(g) => GeometryType::LineString(g),
                Geometry::MultiLineString(g) => GeometryType::MultiLineString(g),
                Geometry::Polygon(g) => GeometryType::Polygon(g),
                Geometry::MultiPolygon(g) => GeometryType::MultiPolygon(g),
                Geometry::GeometryCollection(g) => GeometryType::GeometryCollection(g),
            }
        }
    };
}

impl<T: CoordNum, const N: usize> GeometryTrait for Geometry<[T; N]> {
    geometry_enum_trait_body!();
}

// `&Geometry` is required because `GeometryCollectionTrait::GeometryType<'a>`
// is `&'a Geometry<[T; N]>`.
impl<T: CoordNum, const N: usize> GeometryTrait for &Geometry<[T; N]> {
    geometry_enum_trait_body!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn geometry_enum_dispatch() {
        let mut p: Polygon<[f64; 2]> = Polygon::new();
        p.add_ring([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
        let g: Geometry<[f64; 2]> = Geometry::Polygon(p);
        match g.as_type() {
            GeometryType::Polygon(_) => {}
            _ => panic!("expected Polygon variant"),
        }
    }

    #[test]
    fn geometry_collection_dispatch() {
        let mut p: Polygon<[f64; 2]> = Polygon::new();
        p.add_ring([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
        let gc: GeometryCollection<[f64; 2]> =
            GeometryCollection::from_vec(vec![Geometry::Polygon(p)]);
        let g: Geometry<[f64; 2]> = Geometry::GeometryCollection(gc);

        match g.as_type() {
            GeometryType::GeometryCollection(c) => {
                assert_eq!(GeometryCollectionTrait::num_geometries(c), 1);
                let inner = GeometryCollectionTrait::geometry(c, 0).unwrap();
                match GeometryTrait::as_type(&inner) {
                    GeometryType::Polygon(_) => {}
                    _ => panic!("expected nested Polygon"),
                }
            }
            _ => panic!("expected GeometryCollection variant"),
        }
    }

    #[test]
    fn geometry_collection_ref_dispatch() {
        let mut p: Polygon<[f64; 2]> = Polygon::new();
        p.add_ring([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
        let gc: GeometryCollection<[f64; 2]> =
            GeometryCollection::from_vec(vec![Geometry::Polygon(p)]);
        let gc_ref = &gc;

        assert_eq!(GeometryCollectionTrait::num_geometries(&gc_ref), 1);
        let inner = GeometryCollectionTrait::geometry(&gc_ref, 0).unwrap();
        match GeometryTrait::as_type(&inner) {
            GeometryType::Polygon(_) => {}
            _ => panic!("expected nested Polygon"),
        }
    }
}
