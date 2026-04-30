use geo_traits::PolygonTrait;

use crate::geo_traits::geom_trait_body;
use crate::{CoordNum, LineStringView, Polygon, PolygonRef, PolygonView};

macro_rules! polygon_trait_body {
    () => {
        type RingType<'a>
            = LineStringView<'a, [T; N]>
        where
            Self: 'a;
        fn exterior(&self) -> Option<Self::RingType<'_>> {
            if self.is_empty() {
                None
            } else {
                // FQ disambiguates with `PolygonTrait::exterior` (which we're
                // implementing). The `&R: PolygonRef` blanket impl ensures
                // this works uniformly across owned, view, and `&owned`.
                Some(PolygonRef::exterior(self))
            }
        }
        fn num_interiors(&self) -> usize {
            self.len().saturating_sub(1)
        }
        unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
            let r = self.ring_offsets();
            let s = r[i + 1] as usize;
            let e = r[i + 2] as usize;
            LineStringView::from_raw(unsafe { self.raw_coords().get_unchecked(s..e) })
        }
    };
}

impl<T: CoordNum, const N: usize> geo_traits::GeometryTrait for Polygon<[T; N]> {
    geom_trait_body!(Polygon, Polygon = Polygon<[T; N]>, dim = N);
}

impl<T: CoordNum, const N: usize> PolygonTrait for Polygon<[T; N]> {
    polygon_trait_body!();
}

impl<'g, T: CoordNum + 'g, const N: usize> geo_traits::GeometryTrait for &'g Polygon<[T; N]> {
    geom_trait_body!(Polygon, Polygon = Polygon<[T; N]>, dim = N);
}

impl<'g, T: CoordNum + 'g, const N: usize> PolygonTrait for &'g Polygon<[T; N]> {
    polygon_trait_body!();
}

impl<'v, T: CoordNum, const N: usize> geo_traits::GeometryTrait for PolygonView<'v, [T; N]> {
    geom_trait_body!(Polygon, Polygon = PolygonView<'v, [T; N]>, dim = N);
}

impl<'v, T: CoordNum, const N: usize> PolygonTrait for PolygonView<'v, [T; N]> {
    polygon_trait_body!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_traits::LineStringTrait;

    #[test]
    fn polygon_via_trait() {
        let mut p: Polygon<[f64; 2]> = Polygon::new();
        p.add_ring([[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]]);
        p.add_ring([[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0]]);

        let ext = PolygonTrait::exterior(&p).unwrap();
        assert_eq!(LineStringTrait::num_coords(&ext), 4);
        assert_eq!(PolygonTrait::num_interiors(&p), 1);
        let int0 = PolygonTrait::interior(&p, 0).unwrap();
        assert_eq!(LineStringTrait::num_coords(&int0), 4);
    }

    #[test]
    fn polygon_ref_via_trait() {
        let mut p: Polygon<[f64; 2]> = Polygon::new();
        p.add_ring([[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]]);
        p.add_ring([[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0]]);

        let p_ref = &p;
        let ext = PolygonTrait::exterior(&p_ref).unwrap();
        assert_eq!(LineStringTrait::num_coords(&ext), 4);
        assert_eq!(PolygonTrait::num_interiors(&p_ref), 1);
        let int0 = PolygonTrait::interior(&p_ref, 0).unwrap();
        assert_eq!(LineStringTrait::num_coords(&int0), 4);
    }
}
