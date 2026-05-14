use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::error::GisError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    pub fn validate(&self) -> Result<(), GisError> {
        if self.x.is_nan() || self.y.is_nan() {
            return Err(GisError::InvalidCoordinate(
                "coordinates cannot be NaN".to_string(),
            ));
        }
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(GisError::InvalidCoordinate(
                "coordinates must be finite".to_string(),
            ));
        }
        Ok(())
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "POINT ({} {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineString {
    pub points: Vec<Point>,
}

impl LineString {
    pub fn new(points: Vec<Point>) -> Result<Self, GisError> {
        if points.len() < 2 {
            return Err(GisError::ParseError(
                "LineString must have at least 2 points".to_string(),
            ));
        }
        Ok(LineString { points })
    }

    pub fn validate(&self) -> Result<(), GisError> {
        for point in &self.points {
            point.validate()?;
        }
        Ok(())
    }
}

impl fmt::Display for LineString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let coords: Vec<String> = self
            .points
            .iter()
            .map(|p| format!("{} {}", p.x, p.y))
            .collect();
        write!(f, "LINESTRING ({})", coords.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub exterior: LineString,
    pub holes: Vec<LineString>,
}

impl Polygon {
    pub fn new(exterior: LineString, holes: Vec<LineString>) -> Result<Self, GisError> {
        if exterior.points.len() < 4 {
            return Err(GisError::RingTooShort);
        }
        let first = exterior.points.first().unwrap();
        let last = exterior.points.last().unwrap();
        if first != last {
            return Err(GisError::RingNotClosed);
        }
        for hole in &holes {
            if hole.points.len() < 4 {
                return Err(GisError::RingTooShort);
            }
            let hole_first = hole.points.first().unwrap();
            let hole_last = hole.points.last().unwrap();
            if hole_first != hole_last {
                return Err(GisError::RingNotClosed);
            }
            hole.validate()?;
        }
        exterior.validate()?;
        Ok(Polygon { exterior, holes })
    }

    pub fn from_points(exterior: Vec<Point>, holes: Vec<Vec<Point>>) -> Result<Self, GisError> {
        let exterior_line = LineString::new(exterior)?;
        let hole_lines: Result<Vec<LineString>, _> =
            holes.into_iter().map(LineString::new).collect();
        Polygon::new(exterior_line, hole_lines?)
    }

    pub fn validate(&self) -> Result<(), GisError> {
        self.exterior.validate()?;
        for hole in &self.holes {
            hole.validate()?;
        }
        Ok(())
    }
}

impl fmt::Display for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ring_fmt = |ring: &LineString| -> String {
            let coords: Vec<String> = ring
                .points
                .iter()
                .map(|p| format!("{} {}", p.x, p.y))
                .collect();
            format!("({})", coords.join(", "))
        };
        let mut rings = vec![ring_fmt(&self.exterior)];
        for hole in &self.holes {
            rings.push(ring_fmt(hole));
        }
        write!(f, "POLYGON ({})", rings.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiPoint {
    pub points: Vec<Point>,
}

impl MultiPoint {
    pub fn new(points: Vec<Point>) -> Result<Self, GisError> {
        if points.is_empty() {
            return Err(GisError::EmptyMultiGeometry);
        }
        for point in &points {
            point.validate()?;
        }
        Ok(MultiPoint { points })
    }

    pub fn validate(&self) -> Result<(), GisError> {
        for point in &self.points {
            point.validate()?;
        }
        Ok(())
    }
}

impl fmt::Display for MultiPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let coords: Vec<String> = self
            .points
            .iter()
            .map(|p| format!("({} {})", p.x, p.y))
            .collect();
        write!(f, "MULTIPOINT ({})", coords.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiLineString {
    pub lines: Vec<LineString>,
}

impl MultiLineString {
    pub fn new(lines: Vec<LineString>) -> Result<Self, GisError> {
        if lines.is_empty() {
            return Err(GisError::EmptyMultiGeometry);
        }
        for line in &lines {
            line.validate()?;
        }
        Ok(MultiLineString { lines })
    }

    pub fn validate(&self) -> Result<(), GisError> {
        for line in &self.lines {
            line.validate()?;
        }
        Ok(())
    }
}

impl fmt::Display for MultiLineString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lines: Vec<String> = self
            .lines
            .iter()
            .map(|line| {
                let coords: Vec<String> = line
                    .points
                    .iter()
                    .map(|p| format!("{} {}", p.x, p.y))
                    .collect();
                format!("({})", coords.join(", "))
            })
            .collect();
        write!(f, "MULTILINESTRING ({})", lines.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiPolygon {
    pub polygons: Vec<Polygon>,
}

impl MultiPolygon {
    pub fn new(polygons: Vec<Polygon>) -> Result<Self, GisError> {
        if polygons.is_empty() {
            return Err(GisError::EmptyMultiGeometry);
        }
        Ok(MultiPolygon { polygons })
    }

    pub fn validate(&self) -> Result<(), GisError> {
        for poly in &self.polygons {
            poly.validate()?;
        }
        Ok(())
    }
}

impl fmt::Display for MultiPolygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let polys: Vec<String> = self.polygons.iter().map(|p| p.to_string()).collect();
        write!(f, "MULTIPOLYGON ({})", polys.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryCollection {
    pub geometries: Vec<Geometry>,
}

impl GeometryCollection {
    pub fn new(geometries: Vec<Geometry>) -> Result<Self, GisError> {
        if geometries.is_empty() {
            return Err(GisError::EmptyGeometryCollection);
        }
        Ok(GeometryCollection { geometries })
    }
}

impl fmt::Display for GeometryCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let geos: Vec<String> = self.geometries.iter().map(|g| g.to_string()).collect();
        write!(f, "GEOMETRYCOLLECTION ({})", geos.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection),
}

impl Geometry {
    pub fn validate(&self) -> Result<(), GisError> {
        match self {
            Geometry::Point(p) => p.validate(),
            Geometry::LineString(l) => l.validate(),
            Geometry::Polygon(p) => {
                p.validate()?;
                for hole in &p.holes {
                    hole.validate()?;
                }
                Ok(())
            }
            Geometry::MultiPoint(m) => m.validate(),
            Geometry::MultiLineString(m) => m.validate(),
            Geometry::MultiPolygon(m) => {
                for poly in &m.polygons {
                    poly.validate()?;
                }
                Ok(())
            }
            Geometry::GeometryCollection(c) => {
                for geo in &c.geometries {
                    geo.validate()?;
                }
                Ok(())
            }
        }
    }

    pub fn from_wkb(data: &[u8]) -> Result<Self, GisError> {
        crate::wkb::parse_wkb(data)
    }
}

impl fmt::Display for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Geometry::Point(p) => write!(f, "{}", p),
            Geometry::LineString(l) => write!(f, "{}", l),
            Geometry::Polygon(p) => write!(f, "{}", p),
            Geometry::MultiPoint(m) => write!(f, "{}", m),
            Geometry::MultiLineString(m) => write!(f, "{}", m),
            Geometry::MultiPolygon(m) => write!(f, "{}", m),
            Geometry::GeometryCollection(c) => write!(f, "{}", c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
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
    fn test_multilinestring_empty() {
        let lines: Vec<LineString> = vec![];
        let ml = MultiLineString::new(lines);
        assert!(ml.is_err());
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
    fn test_multipolygon_empty() {
        let polygons: Vec<Polygon> = vec![];
        let mp = MultiPolygon::new(polygons);
        assert!(mp.is_err());
    }

    #[test]
    fn test_geometry_collection_valid() {
        let point = Geometry::Point(Point::new(0.0, 0.0));
        let line = Geometry::LineString(
            LineString::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)]).unwrap(),
        );
        let gc = GeometryCollection::new(vec![point, line]);
        assert!(gc.is_ok());
    }

    #[test]
    fn test_geometry_collection_empty() {
        let geos: Vec<Geometry> = vec![];
        let gc = GeometryCollection::new(geos);
        assert!(gc.is_err());
    }

    #[test]
    fn test_geometry_display_point() {
        let p = Point::new(1.5, 2.5);
        assert_eq!(format!("{}", p), "POINT (1.5 2.5)");
    }

    #[test]
    fn test_geometry_display_linestring() {
        let points = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
        let ls = LineString::new(points).unwrap();
        assert_eq!(format!("{}", ls), "LINESTRING (0 0, 1 1)");
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
}
