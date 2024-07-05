pub mod reader;
pub mod writer;

use alloc::string::ToString;
use geozero::error::{GeozeroError, Result};
use geozero::GeozeroGeometry;

pub trait ToFlatgeom<const D: usize> {
    fn to_flatgeom(&self) -> Result<crate::Geometry<[f64; D]>>;
}

impl<const D: usize, T: GeozeroGeometry> ToFlatgeom<D> for T {
    fn to_flatgeom(&self) -> Result<crate::Geometry<[f64; D]>> {
        let mut writer = writer::FlatgeomWriter::<D>::new();
        self.process_geom(&mut writer)?;
        writer
            .take_geometry()
            .ok_or(GeozeroError::Geometry("Missing geometry".to_string()))
    }
}
