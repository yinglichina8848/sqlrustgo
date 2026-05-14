use crate::error::GisError;
use crate::geometry::{
    Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};

pub struct WktParser {
    input: Vec<char>,
    pos: usize,
}

impl WktParser {
    pub fn new(wkt: &str) -> Self {
        WktParser {
            input: wkt.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.get(self.pos) {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), GisError> {
        self.skip_whitespace();
        match self.consume() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(GisError::invalid_wkt(format!(
                "expected '{}' but found '{}'",
                expected, ch
            ))),
            None => Err(GisError::invalid_wkt(format!(
                "expected '{}' but reached end of input",
                expected
            ))),
        }
    }

    fn parse_number(&mut self) -> Result<f64, GisError> {
        self.skip_whitespace();
        let mut num_str = String::new();
        while let Some(&ch) = self.input.get(self.pos) {
            if ch.is_numeric() || ch == '-' || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' {
                num_str.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        num_str
            .parse::<f64>()
            .map_err(|_| GisError::invalid_wkt(format!("invalid number: {}", num_str)))
    }

    fn parse_point(&mut self) -> Result<Point, GisError> {
        self.expect_char('(')?;
        let x = self.parse_number()?;
        let y = self.parse_number()?;
        self.expect_char(')')?;
        Ok(Point::new(x, y))
    }

    fn parse_point_coordinates(&mut self) -> Result<Point, GisError> {
        self.skip_whitespace();
        let x = self.parse_number()?;
        let y = self.parse_number()?;
        Ok(Point::new(x, y))
    }

    fn parse_linestring(&mut self) -> Result<LineString, GisError> {
        self.expect_char('(')?;
        self.skip_whitespace();
        if self.peek() == Some(')') {
            self.pos += 1;
            return Err(GisError::EmptyGeometry);
        }
        let mut points = Vec::new();
        loop {
            let point = self.parse_point_coordinates()?;
            points.push(point);
            self.skip_whitespace();
            if self.peek() == Some(')') {
                self.pos += 1;
                break;
            }
            if self.peek() == Some(',') {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(GisError::invalid_wkt("expected ',' or ')'".to_string()));
            }
        }
        LineString::new(points)
    }

    fn parse_polygon_ring(&mut self) -> Result<Vec<Point>, GisError> {
        self.skip_whitespace();
        if self.peek() == Some('(') {
            self.pos += 1;
        }
        self.skip_whitespace();
        if self.peek() == Some(')') {
            self.pos += 1;
            return Err(GisError::RingTooShort);
        }
        let mut points = Vec::new();
        loop {
            self.skip_whitespace();
            let x = self.parse_number()?;
            let y = self.parse_number()?;
            points.push(Point::new(x, y));
            self.skip_whitespace();
            if self.peek() == Some(')') {
                self.pos += 1;
                break;
            }
            if self.peek() == Some(',') {
                self.pos += 1;
            } else {
                return Err(GisError::invalid_wkt("expected ',' or ')'".to_string()));
            }
        }
        Ok(points)
    }

    fn parse_polygon(&mut self) -> Result<Polygon, GisError> {
        self.expect_char('(')?;
        self.skip_whitespace();
        if self.peek() == Some(')') {
            self.pos += 1;
            return Err(GisError::EmptyGeometry);
        }
        let mut rings = Vec::new();
        loop {
            let ring_points = self.parse_polygon_ring()?;
            if ring_points.len() < 4 {
                return Err(GisError::RingTooShort);
            }
            let first = ring_points.first().unwrap();
            let last = ring_points.last().unwrap();
            if first != last {
                return Err(GisError::RingNotClosed);
            }
            rings.push(ring_points);
            self.skip_whitespace();
            if self.peek() == Some(')') {
                self.pos += 1;
                break;
            }
            if self.peek() == Some(',') {
                self.pos += 1;
            } else {
                return Err(GisError::invalid_wkt("expected ',' or ')'".to_string()));
            }
        }
        if rings.is_empty() {
            return Err(GisError::EmptyGeometry);
        }
        let exterior_points = rings.remove(0);
        let exterior = LineString::new(exterior_points)?;
        let holes: Result<Vec<LineString>, _> = rings.into_iter().map(LineString::new).collect();
        Polygon::new(exterior, holes?)
    }

    fn parse_multipoint(&mut self) -> Result<MultiPoint, GisError> {
        self.expect_char('(')?;
        self.skip_whitespace();
        if self.peek() == Some(')') {
            self.pos += 1;
            return Err(GisError::EmptyMultiGeometry);
        }
        let mut points = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek() == Some('(') {
                let point = self.parse_point()?;
                points.push(point);
            } else {
                let point = self.parse_point_coordinates()?;
                points.push(point);
            }
            self.skip_whitespace();
            if self.peek() == Some(')') {
                self.pos += 1;
                break;
            }
            if self.peek() == Some(',') {
                self.pos += 1;
            } else {
                return Err(GisError::invalid_wkt("expected ',' or ')'".to_string()));
            }
        }
        MultiPoint::new(points)
    }

    fn parse_multilinestring(&mut self) -> Result<MultiLineString, GisError> {
        self.expect_char('(')?;
        self.skip_whitespace();
        if self.peek() == Some(')') {
            self.pos += 1;
            return Err(GisError::EmptyMultiGeometry);
        }
        let mut lines = Vec::new();
        loop {
            self.skip_whitespace();
            let line = self.parse_linestring()?;
            lines.push(line);
            self.skip_whitespace();
            if self.peek() == Some(')') {
                self.pos += 1;
                break;
            }
            if self.peek() == Some(',') {
                self.pos += 1;
            } else {
                return Err(GisError::invalid_wkt("expected ',' or ')'".to_string()));
            }
        }
        MultiLineString::new(lines)
    }

    fn parse_multipolygon(&mut self) -> Result<MultiPolygon, GisError> {
        self.expect_char('(')?;
        self.skip_whitespace();
        if self.peek() == Some(')') {
            self.pos += 1;
            return Err(GisError::EmptyMultiGeometry);
        }
        let mut polygons = Vec::new();
        loop {
            self.skip_whitespace();
            let poly = self.parse_polygon()?;
            polygons.push(poly);
            self.skip_whitespace();
            if self.peek() == Some(')') {
                self.pos += 1;
                break;
            }
            if self.peek() == Some(',') {
                self.pos += 1;
            } else {
                return Err(GisError::invalid_wkt("expected ',' or ')'".to_string()));
            }
        }
        MultiPolygon::new(polygons)
    }

    fn parse_geometrycollection(&mut self) -> Result<GeometryCollection, GisError> {
        self.expect_char('(')?;
        self.skip_whitespace();
        if self.peek() == Some(')') {
            self.pos += 1;
            return Err(GisError::EmptyGeometryCollection);
        }
        let mut geometries = Vec::new();
        loop {
            self.skip_whitespace();
            let geo = Self::parse_geometry(self)?;
            geometries.push(geo);
            self.skip_whitespace();
            if self.peek() == Some(')') {
                self.pos += 1;
                break;
            }
            if self.peek() == Some(',') {
                self.pos += 1;
            } else {
                return Err(GisError::invalid_wkt("expected ',' or ')'".to_string()));
            }
        }
        GeometryCollection::new(geometries)
    }

    pub fn parse_geometry(&mut self) -> Result<Geometry, GisError> {
        self.skip_whitespace();
        let word = self.read_word();
        match word.to_uppercase().as_str() {
            "POINT" => {
                let point = self.parse_point()?;
                Ok(Geometry::Point(point))
            }
            "LINESTRING" => {
                let line = self.parse_linestring()?;
                Ok(Geometry::LineString(line))
            }
            "POLYGON" => {
                let poly = self.parse_polygon()?;
                Ok(Geometry::Polygon(poly))
            }
            "MULTIPOINT" => {
                let mp = self.parse_multipoint()?;
                Ok(Geometry::MultiPoint(mp))
            }
            "MULTILINESTRING" => {
                let ml = self.parse_multilinestring()?;
                Ok(Geometry::MultiLineString(ml))
            }
            "MULTIPOLYGON" => {
                let mp = self.parse_multipolygon()?;
                Ok(Geometry::MultiPolygon(mp))
            }
            "GEOMETRYCOLLECTION" => {
                let gc = self.parse_geometrycollection()?;
                Ok(Geometry::GeometryCollection(gc))
            }
            _ => Err(GisError::UnsupportedGeometryType(word)),
        }
    }

    fn read_word(&mut self) -> String {
        self.skip_whitespace();
        let mut word = String::new();
        while let Some(&ch) = self.input.get(self.pos) {
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        word
    }

    pub fn parse(wkt: &str) -> Result<Geometry, GisError> {
        let mut parser = WktParser::new(wkt);
        let result = parser.parse_geometry();
        parser.skip_whitespace();
        if parser.pos < parser.input.len() {
            Err(GisError::invalid_wkt("unexpected text after geometry"))
        } else {
            result
        }
    }
}

pub fn parse_wkt(wkt: &str) -> Result<Geometry, GisError> {
    WktParser::parse(wkt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_point() {
        let result = parse_wkt("POINT (1 2)").unwrap();
        match result {
            Geometry::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
            }
            _ => panic!("expected Point"),
        }
    }

    #[test]
    fn test_parse_point_with_decimals() {
        let result = parse_wkt("POINT (1.5 2.75)").unwrap();
        match result {
            Geometry::Point(p) => {
                assert_eq!(p.x, 1.5);
                assert_eq!(p.y, 2.75);
            }
            _ => panic!("expected Point"),
        }
    }

    #[test]
    fn test_parse_linestring() {
        let result = parse_wkt("LINESTRING (0 0, 1 1, 2 2)").unwrap();
        match result {
            Geometry::LineString(ls) => {
                assert_eq!(ls.points.len(), 3);
            }
            _ => panic!("expected LineString"),
        }
    }

    #[test]
    fn test_parse_polygon() {
        let result = parse_wkt("POLYGON ((0 0, 10 0, 10 10, 0 10, 0 0))").unwrap();
        match result {
            Geometry::Polygon(p) => {
                assert_eq!(p.exterior.points.len(), 5);
                assert!(p.holes.is_empty());
            }
            _ => panic!("expected Polygon"),
        }
    }

    #[test]
    fn test_parse_polygon_with_hole() {
        let wkt = "POLYGON ((0 0, 10 0, 10 10, 0 10, 0 0), (2 2, 2 8, 8 8, 8 2, 2 2))";
        let result = parse_wkt(wkt).unwrap();
        match result {
            Geometry::Polygon(p) => {
                assert_eq!(p.exterior.points.len(), 5);
                assert_eq!(p.holes.len(), 1);
            }
            _ => panic!("expected Polygon"),
        }
    }

    #[test]
    fn test_parse_multipoint() {
        let result = parse_wkt("MULTIPOINT ((0 0), (1 1), (2 2))").unwrap();
        match result {
            Geometry::MultiPoint(mp) => {
                assert_eq!(mp.points.len(), 3);
            }
            _ => panic!("expected MultiPoint"),
        }
    }

    #[test]
    fn test_parse_multipoint_without_parens() {
        let result = parse_wkt("MULTIPOINT (0 0, 1 1, 2 2)").unwrap();
        match result {
            Geometry::MultiPoint(mp) => {
                assert_eq!(mp.points.len(), 3);
            }
            _ => panic!("expected MultiPoint"),
        }
    }

    #[test]
    fn test_parse_multilinestring() {
        let result = parse_wkt("MULTILINESTRING ((0 0, 1 1), (2 2, 3 3))").unwrap();
        match result {
            Geometry::MultiLineString(ml) => {
                assert_eq!(ml.lines.len(), 2);
            }
            _ => panic!("expected MultiLineString"),
        }
    }

    #[test]
    fn test_parse_multipolygon() {
        let result = parse_wkt(
            "MULTIPOLYGON (((0 0, 10 0, 10 10, 0 10, 0 0)), ((20 20, 30 20, 30 30, 20 30, 20 20)))",
        )
        .unwrap();
        match result {
            Geometry::MultiPolygon(mp) => {
                assert_eq!(mp.polygons.len(), 2);
            }
            _ => panic!("expected MultiPolygon"),
        }
    }

    #[test]
    fn test_parse_geometrycollection() {
        let result = parse_wkt("GEOMETRYCOLLECTION (POINT (0 0), LINESTRING (0 0, 1 1))").unwrap();
        match result {
            Geometry::GeometryCollection(gc) => {
                assert_eq!(gc.geometries.len(), 2);
            }
            _ => panic!("expected GeometryCollection"),
        }
    }

    #[test]
    fn test_parse_empty_geometry() {
        let result = parse_wkt("POINT EMPTY");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_wkt() {
        let result = parse_wkt("NOTAGEOMETRY (0 0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_linestring_too_few_points() {
        let result = parse_wkt("LINESTRING (0 0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_polygon_ring_not_closed() {
        let result = parse_wkt("POLYGON ((0 0, 10 0, 10 10, 0 10))");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_polygon_too_few_ring_points() {
        let result = parse_wkt("POLYGON ((0 0, 10 0, 0 0))");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_wkt_preserves_case() {
        let result = parse_wkt("Point (1 2)").unwrap();
        match result {
            Geometry::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
            }
            _ => panic!("expected Point"),
        }
    }
}
