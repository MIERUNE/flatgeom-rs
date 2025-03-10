# flatgeom

[![Test](https://github.com/MIERUNE/flatgeom-rs/actions/workflows/Test.yml/badge.svg)](https://github.com/MIERUNE/flatgeom-rs/actions/workflows/Test.yml)
[![codecov](https://codecov.io/gh/MIERUNE/flatgeom-rs/graph/badge.svg?token=iFJds9bJoo)](https://codecov.io/gh/MIERUNE/flatgeom-rs)

Geospatial geometry primitives (e.g., Polygons, MultiPolygons) that use flat data structures instead of jagged arrays for efficient serialization and deserialization.

## Visual examples of the data structure

### LineString

![LineString](./docs/01_linestring.png)

### Polygon

![Polygon](./docs/02_polygon.png)

### Polygon with a hole

![Polygon with a hole](./docs/03_polygon_with_a_hole.png)

### Polygon with multiple holes

![Polygon with multiple holes](./docs/04_polygon_with_multiple_holes.png)

### MultiPolygon

![MultiPolygon](./docs/05_multipolygon.png)

### MultiPolygon with holes

![MultiPolygon with holes](./docs/06_multipolygon_with_holes.png)

### Multiple polygons, multiple holes

![Multiple polygons, multiple holes](./docs/07_multipolygon_multiple_holes.png)
