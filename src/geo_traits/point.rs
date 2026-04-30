use geo_traits::PointTrait;

use crate::geo_traits::{CoordRef, geom_trait_body};
use crate::{CoordNum, Point, PointRef, PointView};

macro_rules! point_trait_body {
    () => {
        type CoordType<'a>
            = CoordRef<'a, T, N>
        where
            Self: 'a;
        fn coord(&self) -> Option<Self::CoordType<'_>> {
            Some(CoordRef(self.raw_coord()))
        }
    };
}

impl<T: CoordNum, const N: usize> geo_traits::GeometryTrait for Point<[T; N]> {
    geom_trait_body!(Point, Point = Point<[T; N]>, dim = N);
}

impl<T: CoordNum, const N: usize> PointTrait for Point<[T; N]> {
    point_trait_body!();
}

impl<'g, T: CoordNum + 'g, const N: usize> geo_traits::GeometryTrait for &'g Point<[T; N]> {
    geom_trait_body!(Point, Point = Point<[T; N]>, dim = N);
}

impl<'g, T: CoordNum + 'g, const N: usize> PointTrait for &'g Point<[T; N]> {
    point_trait_body!();
}

impl<'v, T: CoordNum, const N: usize> geo_traits::GeometryTrait for PointView<'v, [T; N]> {
    geom_trait_body!(Point, Point = PointView<'v, [T; N]>, dim = N);
}

impl<'v, T: CoordNum, const N: usize> PointTrait for PointView<'v, [T; N]> {
    point_trait_body!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_traits::{CoordTrait, Dimensions, GeometryTrait};

    #[test]
    fn point_via_trait() {
        let p: Point<[f64; 2]> = Point::new([1.5, 2.5]);
        assert_eq!(GeometryTrait::dim(&p), Dimensions::Xy);
        let c = PointTrait::coord(&p).unwrap();
        assert_eq!((c.x(), c.y()), (1.5, 2.5));
    }

    #[test]
    fn point_ref_via_trait() {
        let p: Point<[f64; 2]> = Point::new([1.5, 2.5]);
        let p_ref = &p;
        assert_eq!(GeometryTrait::dim(&p_ref), Dimensions::Xy);
        let c = PointTrait::coord(&p_ref).unwrap();
        assert_eq!((c.x(), c.y()), (1.5, 2.5));
    }

    #[test]
    fn point_3d() {
        let p: Point<[f64; 3]> = Point::new([1.0, 2.0, 3.0]);
        assert_eq!(GeometryTrait::dim(&p), Dimensions::Xyz);
        let c = PointTrait::coord(&p).unwrap();
        assert_eq!(c.nth_or_panic(2), 3.0);
    }

    #[test]
    fn point_view() {
        let coord = [10.0_f64, 20.0];
        let view = PointView::from_raw(&coord);
        let c = PointTrait::coord(&view).unwrap();
        assert_eq!((c.x(), c.y()), (10.0, 20.0));
    }
}
