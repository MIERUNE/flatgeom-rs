use geo_traits::MultiLineStringTrait;

use crate::geo_traits::geom_trait_body;
use crate::{CoordNum, LineStringView, MultiLineString, MultiLineStringRef, MultiLineStringView};

macro_rules! multi_line_string_trait_body {
    () => {
        type InnerLineStringType<'a>
            = LineStringView<'a, [T; N]>
        where
            Self: 'a;
        fn num_line_strings(&self) -> usize {
            self.len()
        }
        unsafe fn line_string_unchecked(&self, i: usize) -> Self::InnerLineStringType<'_> {
            let off = self.line_offsets();
            let s = off[i] as usize;
            let e = off[i + 1] as usize;
            LineStringView::from_raw(unsafe { self.raw_coords().get_unchecked(s..e) })
        }
    };
}

impl<T: CoordNum, const N: usize> geo_traits::GeometryTrait for MultiLineString<[T; N]> {
    geom_trait_body!(MultiLineString, MultiLineString = MultiLineString<[T; N]>, dim = N);
}

impl<T: CoordNum, const N: usize> MultiLineStringTrait for MultiLineString<[T; N]> {
    multi_line_string_trait_body!();
}

impl<'g, T: CoordNum + 'g, const N: usize> geo_traits::GeometryTrait
    for &'g MultiLineString<[T; N]>
{
    geom_trait_body!(
        MultiLineString,
        MultiLineString = MultiLineString<[T; N]>,
        dim = N
    );
}

impl<'g, T: CoordNum + 'g, const N: usize> MultiLineStringTrait for &'g MultiLineString<[T; N]> {
    multi_line_string_trait_body!();
}

impl<'v, T: CoordNum, const N: usize> geo_traits::GeometryTrait
    for MultiLineStringView<'v, [T; N]>
{
    geom_trait_body!(MultiLineString, MultiLineString = MultiLineStringView<'v, [T; N]>, dim = N);
}

impl<'v, T: CoordNum, const N: usize> MultiLineStringTrait for MultiLineStringView<'v, [T; N]> {
    multi_line_string_trait_body!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_traits::LineStringTrait;

    #[test]
    fn multilinestring_via_trait() {
        let mut mls: MultiLineString<[f64; 2]> = MultiLineString::new();
        mls.add_linestring([[0.0, 0.0], [1.0, 1.0]]);
        mls.add_linestring([[2.0, 2.0], [3.0, 3.0], [4.0, 4.0]]);

        assert_eq!(MultiLineStringTrait::num_line_strings(&mls), 2);
        let ls1 = MultiLineStringTrait::line_string(&mls, 1).unwrap();
        assert_eq!(LineStringTrait::num_coords(&ls1), 3);
    }

    #[test]
    fn multilinestring_ref_via_trait() {
        let mut mls: MultiLineString<[f64; 2]> = MultiLineString::new();
        mls.add_linestring([[0.0, 0.0], [1.0, 1.0]]);
        mls.add_linestring([[2.0, 2.0], [3.0, 3.0], [4.0, 4.0]]);

        let mls_ref = &mls;
        assert_eq!(MultiLineStringTrait::num_line_strings(&mls_ref), 2);
        let ls1 = MultiLineStringTrait::line_string(&mls_ref, 1).unwrap();
        assert_eq!(LineStringTrait::num_coords(&ls1), 3);
    }
}
