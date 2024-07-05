use crate::geometry::{Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Polygon};

use geozero::{GeomProcessor, GeozeroGeometry};

impl<const D: usize> GeozeroGeometry for Geometry<'_, [f64; D]> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> geozero::error::Result<()> {
        process_geometry(self, processor, 0)
    }
}

fn process_geometry<const D: usize, P: geozero::GeomProcessor>(
    geom: &Geometry<'_, [f64; D]>,
    processor: &mut P,
    idx: usize,
) -> Result<(), geozero::error::GeozeroError> {
    match geom {
        Geometry::GeometryCollection(geoms) => {
            processor.geometrycollection_begin(geoms.len(), idx)?;
            for (geom_idx, geom) in geoms.iter().enumerate() {
                process_geometry(geom, processor, geom_idx)?; // FIXME: idx
            }
            processor.geometrycollection_end(idx)?;
            Ok(())
        }
        Geometry::MultiPoint(geom) => process_multipoint(geom, processor, idx),
        Geometry::LineString(geom) => process_linestring(geom, processor, idx),
        Geometry::MultiLineString(geom) => process_multilinestring(geom, processor, idx),
        Geometry::Polygon(geom) => process_polygon(geom, processor, idx),
        Geometry::MultiPolygon(geom) => process_multipolygon(geom, processor, idx),
    }
}

impl<const D: usize> geozero::GeozeroGeometry for MultiPoint<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        process_multipoint(self, processor, 0)
    }
}

fn process_multipoint<const D: usize, P: geozero::GeomProcessor>(
    geom: &MultiPoint<'_, [f64; D]>,
    processor: &mut P,
    idx: usize,
) -> Result<(), geozero::error::GeozeroError> {
    processor.multipoint_begin(geom.len(), idx)?;
    for (coord_idx, coord) in geom.iter().enumerate() {
        if processor.multi_dim() && D >= 3 {
            processor.coordinate(
                coord[0],
                coord[1],
                Some(coord[2]),
                None,
                None,
                None,
                coord_idx,
            )?;
        } else {
            processor.xy(coord[0], coord[1], coord_idx)?;
        }
    }
    processor.multipoint_end(idx)?;
    Ok(())
}

impl<const D: usize> geozero::GeozeroGeometry for LineString<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        process_linestring(self, processor, 0)
    }
}

fn process_linestring<const D: usize, P: geozero::GeomProcessor>(
    geom: &LineString<'_, [f64; D]>,
    processor: &mut P,
    idx: usize,
) -> Result<(), geozero::error::GeozeroError> {
    processor.linestring_begin(true, geom.len(), idx)?;
    for (coord_idx, coord) in geom.iter().enumerate() {
        if processor.multi_dim() && D >= 3 {
            processor.coordinate(
                coord[0],
                coord[1],
                Some(coord[2]),
                None,
                None,
                None,
                coord_idx,
            )?;
        } else {
            processor.xy(coord[0], coord[1], coord_idx)?;
        }
    }
    processor.linestring_end(true, idx)?;
    Ok(())
}

impl<const D: usize> geozero::GeozeroGeometry for MultiLineString<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        process_multilinestring(self, processor, 0)
    }
}

fn process_multilinestring<const D: usize, P: geozero::GeomProcessor>(
    geom: &MultiLineString<'_, [f64; D]>,
    processor: &mut P,
    idx: usize,
) -> Result<(), geozero::error::GeozeroError> {
    processor.multilinestring_begin(geom.len(), idx)?;
    for (ls_idx, ls) in geom.iter().enumerate() {
        processor.linestring_begin(false, ls.len(), ls_idx)?;
        for (coord_idx, coord) in ls.iter().enumerate() {
            if processor.multi_dim() && D >= 3 {
                processor.coordinate(
                    coord[0],
                    coord[1],
                    Some(coord[2]),
                    None,
                    None,
                    None,
                    coord_idx,
                )?;
            } else {
                processor.xy(coord[0], coord[1], coord_idx)?;
            }
        }
        processor.linestring_end(false, ls_idx)?;
    }
    processor.multilinestring_end(idx)?;
    Ok(())
}

impl<const D: usize> geozero::GeozeroGeometry for Polygon<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        process_polygon(self, processor, 0)
    }
}

fn process_polygon<const D: usize, P: geozero::GeomProcessor>(
    geom: &Polygon<'_, [f64; D]>,
    processor: &mut P,
    idx: usize,
) -> Result<(), geozero::error::GeozeroError> {
    processor.polygon_begin(true, geom.len(), idx)?;
    for (ls_idx, ls) in geom.rings().enumerate() {
        processor.linestring_begin(false, ls.len(), ls_idx)?;
        for (coord_idx, coord) in ls.iter_closed().enumerate() {
            if processor.multi_dim() && D >= 3 {
                processor.coordinate(
                    coord[0],
                    coord[1],
                    Some(coord[2]),
                    None,
                    None,
                    None,
                    coord_idx,
                )?;
            } else {
                processor.xy(coord[0], coord[1], coord_idx)?;
            }
        }
        processor.linestring_end(false, ls_idx)?;
    }
    processor.polygon_end(true, idx)?;
    Ok(())
}

impl<const D: usize> geozero::GeozeroGeometry for MultiPolygon<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        process_multipolygon(self, processor, 0)
    }
}

fn process_multipolygon<const D: usize, P: geozero::GeomProcessor>(
    geom: &MultiPolygon<'_, [f64; D]>,
    processor: &mut P,
    idx: usize,
) -> Result<(), geozero::error::GeozeroError> {
    processor.multipolygon_begin(geom.len(), idx)?;
    for (poly_idx, poly) in geom.iter().enumerate() {
        processor.polygon_begin(false, poly.len() + 1, poly_idx)?;
        for (ls_idx, ls) in poly.rings().enumerate() {
            processor.linestring_begin(false, ls.len(), ls_idx)?;
            for (coord_idx, coord) in ls.iter_closed().enumerate() {
                if processor.multi_dim() && D >= 3 {
                    processor.coordinate(
                        coord[0],
                        coord[1],
                        Some(coord[2]),
                        None,
                        None,
                        None,
                        coord_idx,
                    )?;
                } else {
                    processor.xy(coord[0], coord[1], coord_idx)?;
                }
            }
            processor.linestring_end(false, ls_idx)?;
        }
        processor.polygon_end(false, poly_idx)?;
    }
    processor.multipolygon_end(idx)?;
    Ok(())
}
