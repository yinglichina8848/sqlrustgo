use crate::error::GisError;
use crate::geometry::{
    Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ByteOrder {
    BigEndian,
    LittleEndian,
}

pub struct WkbReader<'a> {
    data: &'a [u8],
    pos: usize,
    byte_order: ByteOrder,
}

impl<'a> WkbReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        WkbReader {
            data,
            pos: 0,
            byte_order: ByteOrder::LittleEndian,
        }
    }

    fn read_byte(&mut self) -> Result<u8, GisError> {
        if self.pos >= self.data.len() {
            return Err(GisError::InvalidWkb("unexpected end of data".to_string()));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_u32(&mut self) -> Result<u32, GisError> {
        if self.pos + 4 > self.data.len() {
            return Err(GisError::InvalidWkb("unexpected end of data".to_string()));
        }
        let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        match self.byte_order {
            ByteOrder::BigEndian => Ok(u32::from_be_bytes(bytes)),
            ByteOrder::LittleEndian => Ok(u32::from_le_bytes(bytes)),
        }
    }

    fn read_f64(&mut self) -> Result<f64, GisError> {
        if self.pos + 8 > self.data.len() {
            return Err(GisError::InvalidWkb("unexpected end of data".to_string()));
        }
        let bytes: [u8; 8] = self.data[self.pos..self.pos + 8].try_into().unwrap();
        self.pos += 8;
        match self.byte_order {
            ByteOrder::BigEndian => Ok(f64::from_be_bytes(bytes)),
            ByteOrder::LittleEndian => Ok(f64::from_le_bytes(bytes)),
        }
    }

    fn read_point(&mut self) -> Result<Point, GisError> {
        let x = self.read_f64()?;
        let y = self.read_f64()?;
        let point = Point::new(x, y);
        point.validate()?;
        Ok(point)
    }

    fn read_linestring(&mut self) -> Result<LineString, GisError> {
        let num_points = self.read_u32()? as usize;
        if num_points < 2 {
            return Err(GisError::ParseError(
                "LineString must have at least 2 points".to_string(),
            ));
        }
        let mut points = Vec::with_capacity(num_points);
        for _ in 0..num_points {
            points.push(self.read_point()?);
        }
        LineString::new(points)
    }

    fn read_polygon(&mut self) -> Result<Polygon, GisError> {
        let num_rings = self.read_u32()? as usize;
        if num_rings == 0 {
            return Err(GisError::EmptyGeometry);
        }
        let mut rings = Vec::with_capacity(num_rings);
        for _ in 0..num_rings {
            let num_points = self.read_u32()? as usize;
            if num_points < 4 {
                return Err(GisError::RingTooShort);
            }
            let mut points = Vec::with_capacity(num_points);
            for _ in 0..num_points {
                points.push(self.read_point()?);
            }
            let first = points.first().unwrap();
            let last = points.last().unwrap();
            if first != last {
                return Err(GisError::RingNotClosed);
            }
            rings.push(points);
        }
        let exterior_points = rings.remove(0);
        let exterior = LineString::new(exterior_points)?;
        let holes: Result<Vec<LineString>, _> = rings.into_iter().map(LineString::new).collect();
        Polygon::new(exterior, holes?)
    }

    fn read_multipoint(&mut self) -> Result<MultiPoint, GisError> {
        let num_points = self.read_u32()? as usize;
        if num_points == 0 {
            return Err(GisError::EmptyMultiGeometry);
        }
        let mut points = Vec::with_capacity(num_points);
        for _ in 0..num_points {
            self.expect_wkb_header(1)?;
            points.push(self.read_point()?);
        }
        MultiPoint::new(points)
    }

    fn read_multilinestring(&mut self) -> Result<MultiLineString, GisError> {
        let num_lines = self.read_u32()? as usize;
        if num_lines == 0 {
            return Err(GisError::EmptyMultiGeometry);
        }
        let mut lines = Vec::with_capacity(num_lines);
        for _ in 0..num_lines {
            self.expect_wkb_header(2)?;
            lines.push(self.read_linestring()?);
        }
        MultiLineString::new(lines)
    }

    fn read_multipolygon(&mut self) -> Result<MultiPolygon, GisError> {
        let num_polys = self.read_u32()? as usize;
        if num_polys == 0 {
            return Err(GisError::EmptyMultiGeometry);
        }
        let mut polygons = Vec::with_capacity(num_polys);
        for _ in 0..num_polys {
            self.expect_wkb_header(3)?;
            polygons.push(self.read_polygon()?);
        }
        MultiPolygon::new(polygons)
    }

    fn read_geometrycollection(&mut self) -> Result<GeometryCollection, GisError> {
        let num_geometries = self.read_u32()? as usize;
        if num_geometries == 0 {
            return Err(GisError::EmptyGeometryCollection);
        }
        let mut geometries = Vec::with_capacity(num_geometries);
        for _ in 0..num_geometries {
            let geo = self.read_geometry_inner()?;
            geometries.push(geo);
        }
        GeometryCollection::new(geometries)
    }

    fn expect_wkb_header(&mut self, expected_type: u32) -> Result<(), GisError> {
        let byte_order = self.read_byte()?;
        self.byte_order = if byte_order == 1 {
            ByteOrder::BigEndian
        } else {
            ByteOrder::LittleEndian
        };
        let wkb_type = self.read_u32()?;
        if wkb_type != expected_type {
            return Err(GisError::InvalidWkb(format!(
                "expected WKB type {} but got {}",
                expected_type, wkb_type
            )));
        }
        Ok(())
    }

    fn read_geometry_inner(&mut self) -> Result<Geometry, GisError> {
        let initial_byte_order = self.read_byte()?;
        let byte_order = if initial_byte_order == 1 {
            ByteOrder::BigEndian
        } else {
            ByteOrder::LittleEndian
        };
        self.byte_order = byte_order;
        let wkb_type = self.read_u32()?;
        match wkb_type {
            1 => {
                let point = self.read_point()?;
                Ok(Geometry::Point(point))
            }
            2 => {
                self.pos -= 5;
                self.byte_order = byte_order;
                self.read_byte()?;
                self.read_u32()?;
                let line = self.read_linestring()?;
                Ok(Geometry::LineString(line))
            }
            3 => {
                self.pos -= 5;
                self.byte_order = byte_order;
                self.read_byte()?;
                self.read_u32()?;
                let poly = self.read_polygon()?;
                Ok(Geometry::Polygon(poly))
            }
            4 => {
                self.pos -= 5;
                self.byte_order = byte_order;
                self.read_byte()?;
                self.read_u32()?;
                let mp = self.read_multipoint()?;
                Ok(Geometry::MultiPoint(mp))
            }
            5 => {
                self.pos -= 5;
                self.byte_order = byte_order;
                self.read_byte()?;
                self.read_u32()?;
                let ml = self.read_multilinestring()?;
                Ok(Geometry::MultiLineString(ml))
            }
            6 => {
                self.pos -= 5;
                self.byte_order = byte_order;
                self.read_byte()?;
                self.read_u32()?;
                let mp = self.read_multipolygon()?;
                Ok(Geometry::MultiPolygon(mp))
            }
            7 => {
                self.pos -= 5;
                self.byte_order = byte_order;
                self.read_byte()?;
                self.read_u32()?;
                let gc = self.read_geometrycollection()?;
                Ok(Geometry::GeometryCollection(gc))
            }
            _ => Err(GisError::UnsupportedGeometryType(format!(
                "WKB type {}",
                wkb_type
            ))),
        }
    }

    pub fn read_geometry(&mut self) -> Result<Geometry, GisError> {
        if self.data.is_empty() {
            return Err(GisError::InvalidWkb("empty data".to_string()));
        }
        let result = self.read_geometry_inner()?;
        Ok(result)
    }
}

pub fn parse_wkb(data: &[u8]) -> Result<Geometry, GisError> {
    let mut reader = WkbReader::new(data);
    reader.read_geometry()
}

pub struct WkbWriter {
    byte_order: u8,
}

impl WkbWriter {
    pub fn new() -> Self {
        WkbWriter { byte_order: 0 }
    }

    pub fn with_byte_order(byte_order: u8) -> Self {
        WkbWriter { byte_order }
    }

    fn write_byte(&self, buf: &mut Vec<u8>, val: u8) {
        buf.push(val);
    }

    fn write_u32(&self, buf: &mut Vec<u8>, val: u32) {
        match self.byte_order {
            1 => buf.extend_from_slice(&val.to_be_bytes()),
            _ => buf.extend_from_slice(&val.to_le_bytes()),
        }
    }

    fn write_f64(&self, buf: &mut Vec<u8>, val: f64) {
        match self.byte_order {
            1 => buf.extend_from_slice(&val.to_be_bytes()),
            _ => buf.extend_from_slice(&val.to_le_bytes()),
        }
    }

    fn write_point(&self, buf: &mut Vec<u8>, point: &Point) {
        self.write_f64(buf, point.x);
        self.write_f64(buf, point.y);
    }

    fn write_geometry(&self, buf: &mut Vec<u8>, geometry: &Geometry) {
        match geometry {
            Geometry::Point(p) => {
                self.write_byte(buf, self.byte_order);
                self.write_u32(buf, 1);
                self.write_point(buf, p);
            }
            Geometry::LineString(ls) => {
                self.write_byte(buf, self.byte_order);
                self.write_u32(buf, 2);
                self.write_u32(buf, ls.points.len() as u32);
                for p in &ls.points {
                    self.write_point(buf, p);
                }
            }
            Geometry::Polygon(poly) => {
                self.write_byte(buf, self.byte_order);
                self.write_u32(buf, 3);
                self.write_u32(buf, (1 + poly.holes.len()) as u32);
                self.write_ring(buf, &poly.exterior.points);
                for hole in &poly.holes {
                    self.write_ring(buf, &hole.points);
                }
            }
            Geometry::MultiPoint(mp) => {
                self.write_byte(buf, self.byte_order);
                self.write_u32(buf, 4);
                self.write_u32(buf, mp.points.len() as u32);
                for p in &mp.points {
                    self.write_byte(buf, self.byte_order);
                    self.write_u32(buf, 1);
                    self.write_point(buf, p);
                }
            }
            Geometry::MultiLineString(ml) => {
                self.write_byte(buf, self.byte_order);
                self.write_u32(buf, 5);
                self.write_u32(buf, ml.lines.len() as u32);
                for line in &ml.lines {
                    self.write_byte(buf, self.byte_order);
                    self.write_u32(buf, 2);
                    self.write_u32(buf, line.points.len() as u32);
                    for p in &line.points {
                        self.write_point(buf, p);
                    }
                }
            }
            Geometry::MultiPolygon(mp) => {
                self.write_byte(buf, self.byte_order);
                self.write_u32(buf, 6);
                self.write_u32(buf, mp.polygons.len() as u32);
                for poly in &mp.polygons {
                    self.write_byte(buf, self.byte_order);
                    self.write_u32(buf, 3);
                    self.write_u32(buf, (1 + poly.holes.len()) as u32);
                    self.write_ring(buf, &poly.exterior.points);
                    for hole in &poly.holes {
                        self.write_ring(buf, &hole.points);
                    }
                }
            }
            Geometry::GeometryCollection(gc) => {
                self.write_byte(buf, self.byte_order);
                self.write_u32(buf, 7);
                self.write_u32(buf, gc.geometries.len() as u32);
                for g in &gc.geometries {
                    self.write_geometry(buf, g);
                }
            }
        }
    }

    fn write_ring(&self, buf: &mut Vec<u8>, points: &[Point]) {
        self.write_u32(buf, points.len() as u32);
        for p in points {
            self.write_point(buf, p);
        }
    }

    pub fn write(&self, geometry: &Geometry) -> Vec<u8> {
        let mut buf = Vec::new();
        self.write_geometry(&mut buf, geometry);
        buf
    }
}

impl Default for WkbWriter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn to_wkb(geometry: &Geometry) -> Vec<u8> {
    let writer = WkbWriter::new();
    writer.write(geometry)
}

pub fn to_wkb_big_endian(geometry: &Geometry) -> Vec<u8> {
    let writer = WkbWriter::with_byte_order(1);
    writer.write(geometry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wkb_point() {
        let point = Point::new(1.0, 2.0);
        let geometry = Geometry::Point(point);
        let wkb = to_wkb(&geometry);
        let result = parse_wkb(&wkb).unwrap();
        match result {
            Geometry::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
            }
            _ => panic!("expected Point"),
        }
    }

    #[test]
    fn test_parse_wkb_linestring() {
        let ls = LineString::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)]).unwrap();
        let geometry = Geometry::LineString(ls);
        let wkb = to_wkb(&geometry);
        let result = parse_wkb(&wkb).unwrap();
        match result {
            Geometry::LineString(ls) => {
                assert_eq!(ls.points.len(), 2);
            }
            _ => panic!("expected LineString"),
        }
    }

    #[test]
    fn test_parse_wkb_polygon() {
        let exterior = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
            Point::new(0.0, 0.0),
        ];
        let poly = Polygon::from_points(exterior, vec![]).unwrap();
        let geometry = Geometry::Polygon(poly);
        let wkb = to_wkb(&geometry);
        let result = parse_wkb(&wkb).unwrap();
        match result {
            Geometry::Polygon(p) => {
                assert_eq!(p.exterior.points.len(), 5);
                assert!(p.holes.is_empty());
            }
            _ => panic!("expected Polygon"),
        }
    }

    #[test]
    fn test_to_wkb_from_wkb_point() {
        let point = Point::new(1.0, 2.0);
        let geometry = Geometry::Point(point.clone());
        let wkb = to_wkb(&geometry);
        let parsed = parse_wkb(&wkb).unwrap();
        match parsed {
            Geometry::Point(p) => {
                assert_eq!(p.x, point.x);
                assert_eq!(p.y, point.y);
            }
            _ => panic!("expected Point"),
        }
    }

    #[test]
    fn test_to_wkb_from_wkb_linestring() {
        let ls = LineString::new(vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)]).unwrap();
        let geometry = Geometry::LineString(ls.clone());
        let wkb = to_wkb(&geometry);
        let parsed = parse_wkb(&wkb).unwrap();
        match parsed {
            Geometry::LineString(parsed_ls) => {
                assert_eq!(parsed_ls.points.len(), ls.points.len());
            }
            _ => panic!("expected LineString"),
        }
    }

    #[test]
    fn test_wkb_big_endian() {
        let point = Point::new(1.0, 2.0);
        let geometry = Geometry::Point(point);
        let wkb_be = to_wkb_big_endian(&geometry);
        assert_eq!(wkb_be[0], 0x01);
        let parsed = parse_wkb(&wkb_be).unwrap();
        match parsed {
            Geometry::Point(p) => {
                assert_eq!(p.x, 1.0);
                assert_eq!(p.y, 2.0);
            }
            _ => panic!("expected Point"),
        }
    }

    #[test]
    fn test_parse_empty_multipoint() {
        let wkb_data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let result = parse_wkb(&wkb_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_wkb_type() {
        let wkb_data: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00];
        let result = parse_wkb(&wkb_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_truncated_wkb() {
        let wkb_data: Vec<u8> = vec![0x01, 0x00];
        let result = parse_wkb(&wkb_data);
        assert!(result.is_err());
    }
}
