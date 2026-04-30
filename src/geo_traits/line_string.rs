use geo_traits::LineStringTrait;

use crate::geo_traits::{CoordRef, geom_trait_body};
use crate::{CoordNum, LineString, LineStringRef, LineStringView};

macro_rules! line_string_trait_body {
    () => {
        type CoordType<'a>
            = CoordRef<'a, T, N>
        where
            Self: 'a;
        fn num_coords(&self) -> usize {
            self.raw_coords().len()
        }
        unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
            CoordRef(unsafe { self.raw_coords().get_unchecked(i) })
        }
    };
}

impl<T: CoordNum, const N: usize> geo_traits::GeometryTrait for LineString<[T; N]> {
    geom_trait_body!(LineString, LineString = LineString<[T; N]>, dim = N);
}

impl<T: CoordNum, const N: usize> LineStringTrait for LineString<[T; N]> {
    line_string_trait_body!();
}

impl<'g, T: CoordNum + 'g, const N: usize> geo_traits::GeometryTrait for &'g LineString<[T; N]> {
    geom_trait_body!(LineString, LineString = LineString<[T; N]>, dim = N);
}

impl<'g, T: CoordNum + 'g, const N: usize> LineStringTrait for &'g LineString<[T; N]> {
    line_string_trait_body!();
}

impl<'v, T: CoordNum, const N: usize> geo_traits::GeometryTrait for LineStringView<'v, [T; N]> {
    geom_trait_body!(LineString, LineString = LineStringView<'v, [T; N]>, dim = N);
}

impl<'v, T: CoordNum, const N: usize> LineStringTrait for LineStringView<'v, [T; N]> {
    line_string_trait_body!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use geo_traits::{CoordTrait, Dimensions, GeometryTrait};

    #[test]
    fn linestring_2d() {
        let ls: LineString<[f64; 2]> = LineString::from_raw(vec![[0.0, 0.0], [1.0, 2.0]]);
        assert_eq!(GeometryTrait::dim(&ls), Dimensions::Xy);
        assert_eq!(LineStringTrait::num_coords(&ls), 2);
        let c = ls.coord(1).unwrap();
        assert_eq!((c.x(), c.y()), (1.0, 2.0));
    }

    #[test]
    fn linestring_ref() {
        let ls: LineString<[f64; 2]> = LineString::from_raw(vec![[0.0, 0.0], [1.0, 2.0]]);
        let ls_ref = &ls;
        assert_eq!(GeometryTrait::dim(&ls_ref), Dimensions::Xy);
        assert_eq!(LineStringTrait::num_coords(&ls_ref), 2);
        let c = ls_ref.coord(1).unwrap();
        assert_eq!((c.x(), c.y()), (1.0, 2.0));
    }

    #[test]
    fn linestring_4d_is_xyzm() {
        let ls: LineString<[f64; 4]> =
            LineString::from_raw(vec![[1.0, 2.0, 3.0, 4.0], [5.0, 6.0, 7.0, 8.0]]);
        assert_eq!(GeometryTrait::dim(&ls), Dimensions::Xyzm);
        let c = ls.coord(0).unwrap();
        assert_eq!(c.nth_or_panic(2), 3.0);
        assert_eq!(c.nth_or_panic(3), 4.0);
    }

    #[test]
    fn linestring_5d_is_unknown() {
        let ls: LineString<[f64; 5]> = LineString::from_raw(vec![[1.0, 2.0, 3.0, 4.0, 5.0]]);
        assert_eq!(GeometryTrait::dim(&ls), Dimensions::Unknown(5));
    }

    #[test]
    fn linestring_view() {
        let coords = [[10.0_f64, 20.0], [30.0, 40.0]];
        let view = LineStringView::from_raw(&coords);
        assert_eq!(LineStringTrait::num_coords(&view), 2);
        let c = view.coord(0).unwrap();
        assert_eq!((c.x(), c.y()), (10.0, 20.0));
    }
}
