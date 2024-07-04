use crate::geometry::{Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Polygon};

use geozero::{GeomProcessor, GeozeroGeometry};

impl<const D: usize> GeozeroGeometry for Geometry<'_, [f64; D]> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> geozero::error::Result<()> {
        match self {
            Geometry::MultiPoint(geom) => geom.process_geom(processor),
            Geometry::LineString(geom) => geom.process_geom(processor),
            Geometry::MultiLineString(geom) => geom.process_geom(processor),
            Geometry::Polygon(geom) => geom.process_geom(processor),
            Geometry::MultiPolygon(geom) => geom.process_geom(processor),
        }
    }
}

impl<const D: usize> geozero::GeozeroGeometry for MultiPoint<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        let idx = 0;
        processor.multipoint_begin(self.len(), idx)?;
        for point in self.iter() {
            if processor.multi_dim() && D >= 3 {
                processor.coordinate(point[0], point[1], Some(point[2]), None, None, None, idx)?;
            } else {
                processor.xy(point[0], point[1], idx)?;
            }
        }
        processor.multipoint_end(idx)?;
        Ok(())
    }
}

impl<const D: usize> geozero::GeozeroGeometry for LineString<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        let idx = 0;
        processor.linestring_begin(true, self.len(), idx)?;
        for point in self.iter() {
            if processor.multi_dim() && D >= 3 {
                processor.coordinate(point[0], point[1], Some(point[2]), None, None, None, idx)?;
            } else {
                processor.xy(point[0], point[1], idx)?;
            }
        }
        processor.linestring_end(true, idx)?;
        Ok(())
    }
}

impl<const D: usize> geozero::GeozeroGeometry for MultiLineString<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        let idx = 0;
        processor.multilinestring_begin(self.len(), idx)?;
        for ls in self {
            processor.linestring_begin(false, ls.len(), idx)?;
            for point in &ls {
                if processor.multi_dim() && D >= 3 {
                    processor.coordinate(
                        point[0],
                        point[1],
                        Some(point[2]),
                        None,
                        None,
                        None,
                        idx,
                    )?;
                } else {
                    processor.xy(point[0], point[1], idx)?;
                }
            }
            processor.linestring_end(false, idx)?;
        }
        processor.multilinestring_end(idx)?;
        Ok(())
    }
}

impl<const D: usize> geozero::GeozeroGeometry for Polygon<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        let idx = 0;
        processor.polygon_begin(true, self.len(), idx)?;
        for ls in self.rings() {
            processor.linestring_begin(false, ls.len(), idx)?;
            for point in ls.iter_closed() {
                if processor.multi_dim() && D >= 3 {
                    processor.coordinate(
                        point[0],
                        point[1],
                        Some(point[2]),
                        None,
                        None,
                        None,
                        idx,
                    )?;
                } else {
                    processor.xy(point[0], point[1], idx)?;
                }
            }
            processor.linestring_end(false, idx)?;
        }
        processor.polygon_end(true, idx)?;
        Ok(())
    }
}

impl<const D: usize> geozero::GeozeroGeometry for MultiPolygon<'_, [f64; D]> {
    fn process_geom<P: geozero::GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> geozero::error::Result<()> {
        let idx = 0;

        processor.multipolygon_begin(self.len(), idx)?;
        for poly in self.iter() {
            processor.polygon_begin(false, poly.len() + 1, idx)?;
            for ls in poly.rings() {
                processor.linestring_begin(false, ls.len(), idx)?;
                for point in ls.iter_closed() {
                    if processor.multi_dim() && D >= 3 {
                        processor.coordinate(
                            point[0],
                            point[1],
                            Some(point[2]),
                            None,
                            None,
                            None,
                            idx,
                        )?;
                    } else {
                        processor.xy(point[0], point[1], idx)?;
                    }
                }
                processor.linestring_end(false, idx)?;
            }
            processor.polygon_end(false, idx)?;
        }
        processor.multipolygon_end(idx)?;
        Ok(())
    }
}
