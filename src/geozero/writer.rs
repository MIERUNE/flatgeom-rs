use alloc::vec::Vec;

use geozero::error::Result;
use geozero::GeomProcessor;

use crate::{Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Polygon};

pub struct FlatgeomWriter<const D: usize> {
    geoms: Vec<Geometry<'static, [f64; D]>>,
    collections: Vec<Vec<Geometry<'static, [f64; D]>>>, // stack of GeometryCollection (nested)
    current: Option<Geometry<'static, [f64; D]>>,
    coords: Vec<[f64; D]>,
}

impl<const D: usize> FlatgeomWriter<D> {
    pub fn new() -> Self {
        if D < 2 {
            panic!("Dimension must be at least 2")
        }
        Self {
            geoms: Vec::with_capacity(1),
            collections: Vec::with_capacity(1),
            current: None,
            coords: Vec::new(),
        }
    }

    pub fn take_geometry(mut self) -> Option<Geometry<'static, [f64; D]>> {
        match self.geoms.len() {
            0 => None,
            1 => Some(self.geoms.pop().unwrap()),
            _ => Some(Geometry::GeometryCollection(self.geoms)),
        }
    }
}

impl<const D: usize> Default for FlatgeomWriter<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const D: usize> FlatgeomWriter<D> {
    fn start_geometry(&mut self, geom: Geometry<'static, [f64; D]>) {
        match geom {
            Geometry::GeometryCollection(geoms) => {
                self.collections.push(geoms);
            }
            geom => {
                self.current = Some(geom);
            }
        }
    }

    fn finish_coords(&mut self, idx: usize) {
        if let Some(geom) = self.current.as_mut() {
            let iter = self.coords.iter().copied();
            match geom {
                Geometry::MultiPoint(mp) => mp.extend(iter),
                Geometry::LineString(ls) => ls.extend(iter),
                Geometry::MultiLineString(mls) => mls.add_linestring(iter),
                Geometry::Polygon(poly) => poly.add_ring(iter),
                Geometry::MultiPolygon(mpoly) => {
                    if idx == 0 {
                        mpoly.add_exterior(iter);
                    } else {
                        mpoly.add_interior(iter);
                    }
                }
                _ => {}
            }
        }
        self.coords.clear();
    }

    fn finish_geometry(&mut self) {
        if let Some(last) = self.collections.last_mut() {
            last.push(self.current.take().unwrap());
        } else {
            self.geoms.push(self.current.take().unwrap());
        }
    }
}

impl<const D: usize> GeomProcessor for FlatgeomWriter<D> {
    fn multi_dim(&self) -> bool {
        D >= 3
    }

    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        let mut coord = [0.; D];
        coord[0] = x;
        coord[1] = y;
        self.coords.push(coord);
        Ok(())
    }

    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) -> Result<()> {
        let mut coord = [0.; D];
        coord[0] = x;
        coord[1] = y;
        if D >= 3 {
            if let Some(z) = z {
                coord[2] = z;
            }
        }
        self.coords.push(coord);
        Ok(())
    }

    fn geometrycollection_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.start_geometry(Geometry::GeometryCollection(Vec::new()));
        Ok(())
    }

    fn geometrycollection_end(&mut self, _idx: usize) -> Result<()> {
        self.finish_geometry();
        Ok(())
    }

    fn multipoint_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.start_geometry(Geometry::MultiPoint(MultiPoint::new()));
        Ok(())
    }

    fn linestring_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.start_geometry(Geometry::LineString(LineString::new()));
        }
        Ok(())
    }

    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.start_geometry(Geometry::MultiLineString(MultiLineString::new()));
        Ok(())
    }

    fn polygon_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.start_geometry(Geometry::Polygon(Polygon::new()));
        }
        Ok(())
    }

    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.start_geometry(Geometry::MultiPolygon(MultiPolygon::new()));
        Ok(())
    }

    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.finish_coords(idx);
        self.finish_geometry();
        Ok(())
    }

    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.finish_coords(idx);
        if tagged {
            self.finish_geometry();
        }
        Ok(())
    }

    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        self.finish_geometry();
        Ok(())
    }

    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if tagged {
            self.finish_geometry();
        }
        Ok(())
    }

    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        self.finish_geometry();
        Ok(())
    }
}
