// GIS Spatial Integration Tests
//! Tests for GIS spatial operations
//! R-S4 Gate: cargo test --test gis_spatial_test

use sqlrustgo_gis::geometry::{
    Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use sqlrustgo_gis::rtree::MBR;
use sqlrustgo_gis::spatial::{
    mbr_contains, mbr_intersects, point_in_polygon, st_contains, st_distance, st_intersects,
};
use sqlrustgo_gis::wkt::parse_wkt;

#[test]
fn test_point_creation() {
    let p = Point::new(1.0, 2.0);
    assert_eq!(p.x, 1.0);
    assert_eq!(p.y, 2.0);
}

#[test]
fn test_point_validate_valid() {
    let p = Point::new(1.0, 2.0);
    assert!(p.validate().is_ok());
}

#[test]
fn test_point_validate_nan() {
    let p = Point::new(f64::NAN, 2.0);
    assert!(p.validate().is_err());
}

#[test]
fn test_point_validate_infinite() {
    let p = Point::new(f64::INFINITY, 2.0);
    assert!(p.validate().is_err());
}

#[test]
fn test_linestring_valid() {
    let points = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
    let ls = LineString::new(points);
    assert!(ls.is_ok());
}

#[test]
fn test_linestring_too_few_points() {
    let points = vec![Point::new(0.0, 0.0)];
    let ls = LineString::new(points);
    assert!(ls.is_err());
}

#[test]
fn test_polygon_valid() {
    let exterior = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
        Point::new(0.0, 0.0),
    ];
    let poly = Polygon::from_points(exterior, vec![]);
    assert!(poly.is_ok());
}

#[test]
fn test_polygon_ring_not_closed() {
    let exterior = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
    ];
    let poly = Polygon::from_points(exterior, vec![]);
    assert!(poly.is_err());
}

#[test]
fn test_multipoint_valid() {
    let points = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
    let mp = MultiPoint::new(points);
    assert!(mp.is_ok());
}

#[test]
fn test_multipoint_empty() {
    let points: Vec<Point> = vec![];
    let mp = MultiPoint::new(points);
    assert!(mp.is_err());
}

#[test]
fn test_multilinestring_valid() {
    let lines = vec![
        LineString::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)]).unwrap(),
        LineString::new(vec![Point::new(2.0, 2.0), Point::new(3.0, 3.0)]).unwrap(),
    ];
    let ml = MultiLineString::new(lines);
    assert!(ml.is_ok());
}

#[test]
fn test_multipolygon_valid() {
    let exterior = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
        Point::new(0.0, 0.0),
    ];
    let poly = Polygon::from_points(exterior, vec![]).unwrap();
    let mp = MultiPolygon::new(vec![poly]);
    assert!(mp.is_ok());
}

#[test]
fn test_geometry_collection_valid() {
    let point = Geometry::Point(Point::new(0.0, 0.0));
    let line = Geometry::LineString(
        LineString::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)]).unwrap(),
    );
    let gc = sqlrustgo_gis::geometry::GeometryCollection::new(vec![point, line]);
    assert!(gc.is_ok());
}

#[test]
fn test_geometry_display_point() {
    let p = Point::new(1.5, 2.5);
    assert_eq!(format!("{}", p), "POINT (1.5 2.5)");
}

#[test]
fn test_geometry_display_polygon() {
    let exterior = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
        Point::new(0.0, 0.0),
    ];
    let poly = Polygon::from_points(exterior, vec![]).unwrap();
    let display = format!("{}", poly);
    assert!(display.starts_with("POLYGON"));
}

#[test]
fn test_wkt_parse_point() {
    let result = parse_wkt("POINT (1 2)");
    assert!(result.is_ok());
    if let Ok(Geometry::Point(p)) = result {
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
    }
}

#[test]
fn test_wkt_parse_linestring() {
    let result = parse_wkt("LINESTRING (0 0, 1 1, 2 2)");
    assert!(result.is_ok());
}

#[test]
fn test_wkt_parse_polygon() {
    let result = parse_wkt("POLYGON ((0 0, 10 0, 10 10, 0 10, 0 0))");
    assert!(result.is_ok());
}

#[test]
fn test_st_contains_geometry() {
    let point = Geometry::Point(Point::new(5.0, 5.0));
    let exterior = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
        Point::new(0.0, 0.0),
    ];
    let poly = Polygon::from_points(exterior, vec![]).unwrap();
    let polygon_geom = Geometry::Polygon(poly);
    assert!(st_contains(&polygon_geom, &point));
}

#[test]
fn test_st_distance_points() {
    let p1 = Geometry::Point(Point::new(0.0, 0.0));
    let p2 = Geometry::Point(Point::new(3.0, 4.0));
    let dist = st_distance(&p1, &p2);
    assert!((dist - 5.0).abs() < 0.001);
}

#[test]
fn test_st_intersects_geometry() {
    let line_points = vec![
        Point::new(-1.0, 5.0),
        Point::new(5.0, 5.0),
        Point::new(11.0, 5.0),
    ];
    let line = Geometry::LineString(LineString::new(line_points).unwrap());
    let exterior = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
        Point::new(0.0, 0.0),
    ];
    let poly = Polygon::from_points(exterior, vec![]).unwrap();
    let polygon_geom = Geometry::Polygon(poly);
    assert!(st_intersects(&line, &polygon_geom));
}

#[test]
fn test_mbr_contains() {
    let outer = MBR::new(0.0, 10.0, 0.0, 10.0);
    let inner = MBR::new(2.0, 8.0, 2.0, 8.0);
    assert!(mbr_contains(&outer, &inner));
}

#[test]
fn test_mbr_intersects() {
    let box1 = MBR::new(0.0, 5.0, 0.0, 5.0);
    let box2 = MBR::new(3.0, 8.0, 3.0, 8.0);
    assert!(mbr_intersects(&box1, &box2));
}

#[test]
fn test_point_in_polygon_inside() {
    let point = Point::new(5.0, 5.0);
    let exterior = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
        Point::new(0.0, 0.0),
    ];
    let poly = Polygon::from_points(exterior, vec![]).unwrap();
    assert!(point_in_polygon(&point, &poly));
}

#[test]
fn test_point_in_polygon_outside() {
    let point = Point::new(15.0, 15.0);
    let exterior = vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
        Point::new(0.0, 0.0),
    ];
    let poly = Polygon::from_points(exterior, vec![]).unwrap();
    assert!(!point_in_polygon(&point, &poly));
}
