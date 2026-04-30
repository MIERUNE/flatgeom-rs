use crate::Coord;

/// Computer-friendly Point: a single coordinate.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T: Coord> {
    coord: T,
}

impl<T: Coord + Default> Default for Point<T> {
    fn default() -> Self {
        Self {
            coord: T::default(),
        }
    }
}

pub type Point2<C = f64> = Point<[C; 2]>;
pub type Point3<C = f64> = Point<[C; 3]>;

/// Read-only borrowed view of a Point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointView<'a, T: Coord> {
    coord: &'a T,
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for PointView<'_, T>
where
    T: Coord + serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("Point", 1)?;
        s.serialize_field("coord", self.coord)?;
        s.end()
    }
}

pub type PointView2<'a, C = f64> = PointView<'a, [C; 2]>;
pub type PointView3<'a, C = f64> = PointView<'a, [C; 3]>;

/// Read-only abstraction over [`Point`] and [`PointView`].
pub trait PointRef<T: Coord> {
    fn raw_coord(&self) -> &T;
}

impl<T: Coord> PointRef<T> for Point<T> {
    fn raw_coord(&self) -> &T {
        &self.coord
    }
}

impl<T: Coord> PointRef<T> for PointView<'_, T> {
    fn raw_coord(&self) -> &T {
        self.coord
    }
}

impl<T: Coord> Point<T> {
    pub const fn new(coord: T) -> Self {
        Self { coord }
    }

    pub fn from_raw(coord: T) -> Self {
        Self { coord }
    }

    pub fn into_raw(self) -> T {
        self.coord
    }
}

impl<T: Coord> From<T> for Point<T> {
    fn from(coord: T) -> Self {
        Self { coord }
    }
}

impl<'a, T: Coord> PointView<'a, T> {
    pub const fn from_raw(coord: &'a T) -> Self {
        Self { coord }
    }

    /// Materialize a self-contained owned `Point` by copying the underlying coord.
    pub fn to_owned(&self) -> Point<T> {
        Point { coord: *self.coord }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_basic() {
        let p: Point2 = Point::new([1.5, 2.5]);
        assert_eq!(p.raw_coord(), &[1.5, 2.5]);
        let raw = p.into_raw();
        assert_eq!(raw, [1.5, 2.5]);
    }

    #[test]
    fn point_view_borrow() {
        let coord = [10.0_f64, 20.0];
        let view = PointView::from_raw(&coord);
        assert_eq!(view.raw_coord(), &[10.0, 20.0]);
        let owned = view.to_owned();
        assert_eq!(owned.raw_coord(), &[10.0, 20.0]);
    }

    #[test]
    fn point_via_ref_trait() {
        let p: Point<[i32; 3]> = Point::new([1, 2, 3]);
        assert_eq!(
            <Point<[i32; 3]> as PointRef<[i32; 3]>>::raw_coord(&p),
            &[1, 2, 3]
        );
    }
}
