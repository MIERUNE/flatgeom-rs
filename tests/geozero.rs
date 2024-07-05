//! Testing mutual conversion between geo_types and our MultiPolygon
use core::panic;

use flatgeom::{
    geozero::ToFlatgeom, Geometry2, LineString2, MultiLineString2, MultiPoint2, MultiPolygon2,
    Polygon2,
};

use geozero::ToGeo;

#[test]
fn multipolygon() {
    let mut mpoly = MultiPolygon2::new();
    // 1st polygon
    let mut poly1 = Polygon2::new();
    poly1.add_ring([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]); // exterior
    poly1.add_ring([[1., 1.], [2., 1.], [2., 2.], [1., 2.]]); // interior
    poly1.add_ring([[3., 3.], [4., 3.], [4., 4.], [3., 4.]]); // interior
    mpoly.push(&poly1);

    // 2nd polygon
    let mut poly2 = Polygon2::new();
    poly2.add_ring([[4., 0.], [7., 0.], [7., 3.], [4., 3.]]); // exterior
    poly2.add_ring([[5., 1.], [6., 1.], [6., 2.], [5., 2.]]); // interior
    mpoly.push(&poly2);

    // 3rd polygon
    let mut poly3 = Polygon2::new();
    poly3.add_ring([[4., 0.], [7., 0.], [7., 3.], [4., 3.]]); // exterior
    mpoly.push(&poly3);

    // Conversion
    let Ok(geo) = mpoly.to_geo() else {
        panic!("Conversion failed");
    };
    match &geo {
        geo_types::Geometry::MultiPolygon(geo_mpoly) => {
            assert_eq!(geo_mpoly.0.len(), 3);
            assert_eq!(geo_mpoly.0[0].exterior().0.len(), 5); // ring must be closed
            assert_eq!(geo_mpoly.0[0].interiors().len(), 2);
        }
        _ => panic!("Geometry type must be MultiPolygon"),
    };

    // Inversion
    let Ok(flat): geozero::error::Result<Geometry2> = geo.to_flatgeom() else {
        panic!("Conversion failed");
    };
    match &flat {
        flatgeom::Geometry::MultiPolygon(mpoly) => {
            assert_eq!(mpoly.len(), 3);
        }
        _ => panic!("MultiPolygon is expected"),
    }
}

#[test]
fn polygon() {
    let mut poly = Polygon2::new();
    poly.add_ring([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]); // exterior
    poly.add_ring([[1., 1.], [2., 1.], [2., 2.], [1., 2.]]); // interior
    poly.add_ring([[3., 3.], [4., 3.], [4., 4.], [3., 4.]]); // interior

    // Conversion
    let Ok(geo) = poly.to_geo() else {
        panic!("Conversion failed");
    };
    match &geo {
        geo_types::Geometry::Polygon(geo_poly) => {
            assert_eq!(geo_poly.exterior().0.len(), 5); // ring must be closed
            assert_eq!(geo_poly.interiors().len(), 2);
        }
        _ => panic!("Geometry type must be Polygon"),
    }

    // Inversion
    let Ok(flat): geozero::error::Result<Geometry2> = geo.to_flatgeom() else {
        panic!("Conversion failed");
    };
    match &flat {
        flatgeom::Geometry::Polygon(poly) => {
            assert_eq!(poly.len(), 3);
        }
        _ => panic!("Polygon is expected"),
    }
}

#[test]
fn multilinestring() {
    let mut mls = MultiLineString2::new();
    mls.add_linestring([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);
    mls.add_linestring([[1., 0.], [5., 0.], [5., 5.], [0., 5.]]);
    mls.add_linestring([[2., 0.], [5., 0.], [5., 5.], [0., 5.]]);

    // Conversion
    let Ok(geo) = mls.to_geo() else {
        panic!("Conversion failed");
    };
    match &geo {
        geo_types::Geometry::MultiLineString(geo_mls) => {
            assert_eq!(geo_mls.0.len(), 3); // ring must be closed
            assert_eq!(geo_mls.0[0].0.len(), 4); // ring must be closed
        }
        _ => panic!("Geometry type must be MultiLineString"),
    }

    // Inversion
    let Ok(flat): geozero::error::Result<Geometry2> = geo.to_flatgeom() else {
        panic!("Conversion failed");
    };
    match &flat {
        flatgeom::Geometry::MultiLineString(poly) => {
            assert_eq!(poly.len(), 3);
        }
        _ => panic!("MultiLineString is expected"),
    }
}

#[test]
fn linestring() {
    // Conversion
    let ls = LineString2::from_raw(vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into());
    let Ok(geo) = ls.to_geo() else {
        panic!("Conversion failed");
    };
    match &geo {
        geo_types::Geometry::LineString(geo_ls) => {
            assert_eq!(geo_ls.0.len(), 4); // ring must be closed
        }
        _ => panic!("Geometry type must be LineString"),
    }

    // Inversion
    let Ok(flat): geozero::error::Result<Geometry2> = geo.to_flatgeom() else {
        panic!("Conversion failed");
    };
    match &flat {
        flatgeom::Geometry::LineString(poly) => {
            assert_eq!(poly.len(), 4);
        }
        _ => panic!("LineString is expected"),
    }
}

#[test]
fn multipoint() {
    let mut mp = MultiPoint2::new();
    mp.extend([[0., 0.], [5., 0.], [5., 5.], [0., 5.]]);

    // Conversion
    let Ok(geo) = mp.to_geo() else {
        panic!("Conversion failed");
    };
    match &geo {
        geo_types::Geometry::MultiPoint(geo_mp) => {
            assert_eq!(geo_mp.len(), 4);
        }
        _ => panic!("Geometry type must be MultiPoint"),
    }

    // Inversion
    let Ok(flat): geozero::error::Result<Geometry2> = geo.to_flatgeom() else {
        panic!("Conversion failed");
    };
    match &flat {
        flatgeom::Geometry::MultiPoint(mp) => {
            assert_eq!(mp.len(), 4);
        }
        _ => panic!("MultiPoint is expected"),
    }
}
