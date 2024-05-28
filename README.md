# flatgeom

A geometry type library using flat structures instead of jagged arrays to represent Polygon, MultiPolygon, etc.

It is useful, for example, when serialization/deserialization performance is important.

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
