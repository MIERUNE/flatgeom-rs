use geo_traits::MultiPolygonTrait;

use crate::geo_traits::geom_trait_body;
use crate::{CoordNum, MultiPolygon, MultiPolygonRef, MultiPolygonView, PolygonView};

macro_rules! multi_polygon_trait_body {
    () => {
        type InnerPolygonType<'a>
            = PolygonView<'a, [T; N]>
        where
            Self: 'a;
        fn num_polygons(&self) -> usize {
            self.len()
        }
        unsafe fn polygon_unchecked(&self, i: usize) -> Self::InnerPolygonType<'_> {
            self.get(i)
        }
    };
}

impl<T: CoordNum, const N: usize> geo_traits::GeometryTrait for MultiPolygon<[T; N]> {
    geom_trait_body!(MultiPolygon, MultiPolygon = MultiPolygon<[T; N]>, dim = N);
}

impl<T: CoordNum, const N: usize> MultiPolygonTrait for MultiPolygon<[T; N]> {
    multi_polygon_trait_body!();
}

impl<'g, T: CoordNum + 'g, const N: usize> geo_traits::GeometryTrait for &'g MultiPolygon<[T; N]> {
    geom_trait_body!(MultiPolygon, MultiPolygon = MultiPolygon<[T; N]>, dim = N);
}

impl<'g, T: CoordNum + 'g, const N: usize> MultiPolygonTrait for &'g MultiPolygon<[T; N]> {
    multi_polygon_trait_body!();
}

impl<'v, T: CoordNum, const N: usize> geo_traits::GeometryTrait for MultiPolygonView<'v, [T; N]> {
    geom_trait_body!(MultiPolygon, MultiPolygon = MultiPolygonView<'v, [T; N]>, dim = N);
}

impl<'v, T: CoordNum, const N: usize> MultiPolygonTrait for MultiPolygonView<'v, [T; N]> {
    multi_polygon_trait_body!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Polygon;
    use geo_traits::{LineStringTrait, PolygonTrait};

    #[test]
    fn multipolygon_via_trait() {
        let mut mp: MultiPolygon<[f64; 2]> = MultiPolygon::new();
        let mut p = Polygon::new();
        p.add_ring([[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]]);
        mp.push(&p);
        let mut p2 = Polygon::new();
        p2.add_ring([[10.0, 10.0], [11.0, 10.0], [11.0, 11.0], [10.0, 11.0]]);
        mp.push(&p2);

        assert_eq!(MultiPolygonTrait::num_polygons(&mp), 2);
        let p0 = MultiPolygonTrait::polygon(&mp, 0).unwrap();
        let ext = PolygonTrait::exterior(&p0).unwrap();
        assert_eq!(LineStringTrait::num_coords(&ext), 4);
    }

    #[test]
    fn multipolygon_ref_via_trait() {
        let mut mp: MultiPolygon<[f64; 2]> = MultiPolygon::new();
        let mut p = Polygon::new();
        p.add_ring([[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]]);
        mp.push(&p);
        let mut p2 = Polygon::new();
        p2.add_ring([[10.0, 10.0], [11.0, 10.0], [11.0, 11.0], [10.0, 11.0]]);
        mp.push(&p2);

        let mp_ref = &mp;
        assert_eq!(MultiPolygonTrait::num_polygons(&mp_ref), 2);
        let p0 = MultiPolygonTrait::polygon(&mp_ref, 0).unwrap();
        let ext = PolygonTrait::exterior(&p0).unwrap();
        assert_eq!(LineStringTrait::num_coords(&ext), 4);
    }
}
