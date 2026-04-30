#[cfg(feature = "serde")]
mod tests {
    use flatgeom::{
        Geometry2, LineString2, LineString3, LineStringRef, LineStringView, MultiLineString2,
        MultiLineStringRef, MultiLineStringView, MultiPoint2, MultiPoint3, MultiPointRef,
        MultiPointView, MultiPolygon2, MultiPolygonRef, MultiPolygonView, Point2, PointRef,
        PointView, Polygon2, PolygonRef, PolygonView,
    };

    #[derive(serde::Serialize, serde::Deserialize)]
    struct MyStruct {
        mpoly: MultiPolygon2,
        poly: Polygon2,
        line: LineString3,
        mline: MultiLineString2,
        mpoint: MultiPoint3,
        geom: Geometry2,
    }

    #[test]
    fn test_serde_serialize_deserialize() {
        let m = MyStruct {
            mpoly: Default::default(),
            poly: Default::default(),
            mline: Default::default(),
            line: Default::default(),
            mpoint: Default::default(),
            geom: Geometry2::MultiPoint(Default::default()),
        };
        let serialized = serde_json::to_string(&m).unwrap();
        let _: MyStruct = serde_json::from_str(&serialized).unwrap();
    }

    #[test]
    fn view_serializes_as_owned_geometry() {
        let coord = [1_i32, 2];
        let point: Point2<i32> =
            serde_json::from_str(&serde_json::to_string(&PointView::from_raw(&coord)).unwrap())
                .unwrap();
        assert_eq!(point.raw_coord(), &[1, 2]);

        let coords = [[1_i32, 2], [3, 4]];
        let line: LineString2<i32> = serde_json::from_str(
            &serde_json::to_string(&LineStringView::from_raw(&coords)).unwrap(),
        )
        .unwrap();
        assert_eq!(line.raw_coords(), &coords);

        let multipoint: MultiPoint2<i32> = serde_json::from_str(
            &serde_json::to_string(&MultiPointView::from_raw(&coords)).unwrap(),
        )
        .unwrap();
        assert_eq!(multipoint.raw_coords(), &coords);
    }

    #[test]
    fn polygon_view_serializes_with_topology_lengths() {
        let coords = [
            [-1_i32, -1],
            [-1, -1],
            [0, 0],
            [1, 0],
            [1, 1],
            [0, 1],
            [2, 2],
            [3, 2],
            [3, 3],
            [2, 3],
            [-1, -1],
        ];
        let ring_offsets = [2, 6, 10];
        let view = PolygonView::from_raw(&coords, &ring_offsets);

        let value = serde_json::to_value(view).unwrap();
        assert_eq!(value["topology"], serde_json::json!([4, 4]));
        assert!(value.get("ring_offsets").is_none());

        let polygon: Polygon2<i32> = serde_json::from_value(value).unwrap();
        assert_eq!(polygon.raw_coords(), &coords[2..10]);
        assert_eq!(polygon.ring_offsets(), &[0, 4, 8]);
    }

    #[test]
    fn multiline_view_serializes_with_topology_lengths() {
        let coords = [[0_i32, 0], [1, 1], [2, 2], [3, 3], [4, 4], [5, 5]];
        let line_offsets = [1, 3, 5];
        let view = MultiLineStringView::from_raw(&coords, &line_offsets);

        let value = serde_json::to_value(view).unwrap();
        assert_eq!(value["topology"], serde_json::json!([2, 2]));
        assert!(value.get("line_offsets").is_none());

        let multiline: MultiLineString2<i32> = serde_json::from_value(value).unwrap();
        assert_eq!(multiline.raw_coords(), &coords[1..5]);
        assert_eq!(multiline.line_offsets(), &[0, 2, 4]);
    }

    #[test]
    fn multipolygon_view_serializes_with_single_topology_vector() {
        let mut mp = MultiPolygon2::<i32>::new();
        let mut first = Polygon2::new();
        first.add_ring([[0, 0], [1, 0], [1, 1], [0, 1]]);
        mp.push(&first);

        let mut second = Polygon2::new();
        second.add_ring([[10, 10], [12, 10], [12, 12], [10, 12]]);
        second.add_ring([[11, 11], [11, 12], [12, 12], [12, 11]]);
        mp.push(&second);

        let view = MultiPolygonView::from_raw(
            mp.raw_coords(),
            mp.ring_offsets(),
            &mp.polygon_offsets()[1..=2],
        );

        let value = serde_json::to_value(view).unwrap();
        assert_eq!(value["topology"], serde_json::json!([1, 2, 4, 4]));
        assert!(value.get("ring_offsets").is_none());
        assert!(value.get("polygon_offsets").is_none());

        let owned: MultiPolygon2<i32> = serde_json::from_value(value).unwrap();
        assert_eq!(owned, view.to_owned());
    }
}
