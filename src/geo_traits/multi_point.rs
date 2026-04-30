use geo_traits::MultiPointTrait;

use crate::geo_traits::geom_trait_body;
use crate::{CoordNum, MultiPoint, MultiPointRef, MultiPointView, PointView};

macro_rules! multi_point_trait_body {
    () => {
        type InnerPointType<'a>
            = PointView<'a, [T; N]>
        where
            Self: 'a;
        fn num_points(&self) -> usize {
            self.len()
        }
        unsafe fn point_unchecked(&self, i: usize) -> Self::InnerPointType<'_> {
            PointView::from_raw(unsafe { self.raw_coords().get_unchecked(i) })
        }
    };
}

impl<T: CoordNum, const N: usize> geo_traits::GeometryTrait for MultiPoint<[T; N]> {
    geom_trait_body!(MultiPoint, MultiPoint = MultiPoint<[T; N]>, dim = N);
}

impl<T: CoordNum, const N: usize> MultiPointTrait for MultiPoint<[T; N]> {
    multi_point_trait_body!();
}

impl<'g, T: CoordNum + 'g, const N: usize> geo_traits::GeometryTrait for &'g MultiPoint<[T; N]> {
    geom_trait_body!(MultiPoint, MultiPoint = MultiPoint<[T; N]>, dim = N);
}

impl<'g, T: CoordNum + 'g, const N: usize> MultiPointTrait for &'g MultiPoint<[T; N]> {
    multi_point_trait_body!();
}

impl<'v, T: CoordNum, const N: usize> geo_traits::GeometryTrait for MultiPointView<'v, [T; N]> {
    geom_trait_body!(MultiPoint, MultiPoint = MultiPointView<'v, [T; N]>, dim = N);
}

impl<'v, T: CoordNum, const N: usize> MultiPointTrait for MultiPointView<'v, [T; N]> {
    multi_point_trait_body!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_traits::{CoordTrait, PointTrait};

    #[test]
    fn multipoint_via_trait() {
        let mut mp: MultiPoint<[f64; 2]> = MultiPoint::new();
        mp.push([1.0, 2.0]);
        mp.push([3.0, 4.0]);
        assert_eq!(MultiPointTrait::num_points(&mp), 2);
        let pt = MultiPointTrait::point(&mp, 1).unwrap();
        let c = PointTrait::coord(&pt).unwrap();
        assert_eq!((c.x(), c.y()), (3.0, 4.0));
    }

    #[test]
    fn multipoint_ref_via_trait() {
        let mut mp: MultiPoint<[f64; 2]> = MultiPoint::new();
        mp.push([1.0, 2.0]);
        mp.push([3.0, 4.0]);
        let mp_ref = &mp;
        assert_eq!(MultiPointTrait::num_points(&mp_ref), 2);
        let pt = MultiPointTrait::point(&mp_ref, 1).unwrap();
        let c = PointTrait::coord(&pt).unwrap();
        assert_eq!((c.x(), c.y()), (3.0, 4.0));
    }
}
