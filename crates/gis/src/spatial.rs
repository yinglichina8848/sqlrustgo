//! Spatial query functions for GIS operations
//!
//! Implements spatial predicates and relationships between geometries:
//! - ST_Contains: true if first geometry contains second
//! - ST_Intersects: true if geometries intersect
//! - ST_Within: true if first geometry is within second
//! - ST_Equals: true if geometries are spatially equal
//! - ST_Distance: distance between geometries
//!
//! Uses MBR-based pre-filtering with R-Tree for efficient spatial queries.

use crate::geometry::{Geometry, LineString, MultiPoint, MultiPolygon, Point, Polygon};
use crate::rtree::{MBR, MBRTrait};

/// Edge represented as two points for polygon operations
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    p1: Point,
    p2: Point,
}

impl Edge {
    /// Create a new edge from two points
    fn new(p1: Point, p2: Point) -> Self {
        Edge { p1, p2 }
    }

    /// Get the midpoint of this edge
    #[allow(dead_code)]
    fn midpoint(&self) -> Point {
        Point::new((self.p1.x + self.p2.x) / 2.0, (self.p1.y + self.p2.y) / 2.0)
    }

    /// Check if a horizontal ray from point intersects this edge
    fn ray_intersects_horizontal(&self, point: &Point) -> bool {
        let Point { x: px, y: py } = *point;
        let (p1x, p1y) = (self.p1.x, self.p1.y);
        let (p2x, p2y) = (self.p2.x, self.p2.y);

        // Check if edge is horizontal
        if (p1y - p2y).abs() < f64::EPSILON {
            return false;
        }

        // Ensure p1y < p2y for consistent handling
        let (lower_x, lower_y, upper_x, upper_y) = if p1y < p2y {
            (p1x, p1y, p2x, p2y)
        } else {
            (p2x, p2y, p1x, p1y)
        };

        // Ray goes to the right (positive x direction) from point
        // Check if point's y is within edge's y range
        if py <= lower_y || py > upper_y {
            return false;
        }

        // Calculate x coordinate where ray intersects the edge's y-level
        let intersect_x = lower_x + (upper_x - lower_x) * (py - lower_y) / (upper_y - lower_y);

        // Intersection is to the right of the point
        intersect_x > px
    }
}

impl Polygon {
    /// Get all edges of the exterior ring
    pub fn exterior_edges(&self) -> Vec<Edge> {
        let points = &self.exterior.points;
        let mut edges = Vec::with_capacity(points.len() - 1);
        for i in 0..points.len() - 1 {
            edges.push(Edge::new(points[i], points[i + 1]));
        }
        edges
    }

    /// Get all edges including holes
    pub fn all_edges(&self) -> Vec<Edge> {
        let mut edges = self.exterior_edges();
        for hole in &self.holes {
            let points = &hole.points;
            for i in 0..points.len() - 1 {
                edges.push(Edge::new(points[i], points[i + 1]));
            }
        }
        edges
    }

    /// Iterator over all edges as references
    pub fn edges(&self) -> EdgesIterator<'_> {
        EdgesIterator {
            polygon: self,
            ring_index: 0,
            edge_index: 0,
        }
    }
}

/// Iterator over polygon edges (exterior + holes)
pub struct EdgesIterator<'a> {
    polygon: &'a Polygon,
    ring_index: usize, // 0 = exterior, 1+ = holes
    edge_index: usize,
}

impl<'a> Iterator for EdgesIterator<'a> {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        let rings: Vec<&LineString> = std::iter::once(&self.polygon.exterior)
            .chain(self.polygon.holes.iter())
            .collect();

        if self.ring_index >= rings.len() {
            return None;
        }

        let ring = rings[self.ring_index];
        let points = &ring.points;

        if self.edge_index >= points.len() - 1 {
            self.ring_index += 1;
            self.edge_index = 0;
            return self.next();
        }

        let edge = Edge::new(points[self.edge_index], points[self.edge_index + 1]);
        self.edge_index += 1;
        Some(edge)
    }
}

/// Point-in-polygon test using ray casting algorithm
///
/// Casts a horizontal ray from the point to infinity and counts intersections
/// with polygon edges. An odd count means the point is inside.
pub fn point_in_polygon(point: &Point, polygon: &Polygon) -> bool {
    let mut intersections = 0;
    for edge in polygon.edges() {
        if edge.ray_intersects_horizontal(point) {
            intersections += 1;
        }
    }
    intersections % 2 == 1
}

/// Check if a point is inside a polygon (including boundary)
pub fn point_in_polygon_inclusive(point: &Point, polygon: &Polygon) -> bool {
    // First check if point is on the boundary
    for edge in polygon.edges() {
        if point_on_segment(point, &edge.p1, &edge.p2) {
            return true;
        }
    }
    point_in_polygon(point, polygon)
}

/// Check if a point lies on a line segment
pub fn point_on_segment(point: &Point, seg_start: &Point, seg_end: &Point) -> bool {
    let Point { x: px, y: py } = *point;
    let (x1, y1) = (seg_start.x, seg_start.y);
    let (x2, y2) = (seg_end.x, seg_end.y);

    // Check if point is collinear using cross product
    let cross = (py - y1) * (x2 - x1) - (px - x1) * (y2 - y1);
    if cross.abs() > f64::EPSILON {
        return false;
    }

    // Check if point is within segment bounds
    let min_x = x1.min(x2);
    let max_x = x1.max(x2);
    let min_y = y1.min(y2);
    let max_y = y1.max(y2);

    px >= min_x - f64::EPSILON && px <= max_x + f64::EPSILON &&
    py >= min_y - f64::EPSILON && py <= max_y + f64::EPSILON
}

/// Minimum bounding rectangle intersection test
pub fn mbr_intersects(a: &MBR, b: &MBR) -> bool {
    a.intersects(b)
}

/// Check if MBR a contains MBR b completely
pub fn mbr_contains(a: &MBR, b: &MBR) -> bool {
    a.contains(b)
}

// ============================================================================
// ST_Contains implementation
// ============================================================================

/// Returns true if the first geometry contains the second.
///
/// A.contains(B) is true iff:
/// - B's MBR is within A's MBR
/// - B is entirely inside A (no part of B extends outside A)
///
/// # Arguments
///
/// * `container` - The geometry that should contain the other
/// * `contained` - The geometry that should be contained
///
/// # Examples
///
/// ```
/// use sqlrustgo_gis::{Geometry, Point, Polygon, LineString, spatial::st_contains};
///
/// let square = Geometry::Polygon(Polygon::from_points(
///     vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0), Point::new(10.0, 10.0), Point::new(0.0, 10.0), Point::new(0.0, 0.0)],
///     vec![],
/// ).unwrap());
///
/// let inner_point = Geometry::Point(Point::new(5.0, 5.0));
/// assert!(st_contains(&square, &inner_point));
///
/// let outer_point = Geometry::Point(Point::new(15.0, 15.0));
/// assert!(!st_contains(&square, &outer_point));
/// ```
pub fn st_contains(container: &Geometry, contained: &Geometry) -> bool {
    // MBR pre-filter: contained's MBR must be within container's MBR
    let container_mbr = container.mbr();
    let contained_mbr = contained.mbr();

    if !container_mbr.contains(&contained_mbr) {
        return false;
    }

    // Precise geometric check
    match (container, contained) {
        // Point contains Point
        (Geometry::Point(c), Geometry::Point(p)) => {
            c.x == p.x && c.y == p.y
        }

        // Point contains MultiPoint
        (Geometry::Point(c), Geometry::MultiPoint(mp)) => {
            mp.points.iter().all(|p| p.x == c.x && p.y == c.y)
        }

        // Polygon contains Point
        (Geometry::Polygon(poly), Geometry::Point(pt)) => {
            point_in_polygon_inclusive(pt, poly)
        }

        // Polygon contains MultiPoint
        (Geometry::Polygon(poly), Geometry::MultiPoint(mp)) => {
            mp.points.iter().all(|pt| point_in_polygon_inclusive(pt, poly))
        }

        // Polygon contains LineString
        (Geometry::Polygon(poly), Geometry::LineString(ls)) => {
            ls.points.iter().all(|pt| point_in_polygon_inclusive(pt, poly))
        }

        // Polygon contains Polygon
        (Geometry::Polygon(outer), Geometry::Polygon(inner)) => {
            // All vertices of inner must be in outer
            if !inner.exterior.points.iter().all(|pt| point_in_polygon_inclusive(pt, outer)) {
                return false;
            }
            // Inner must not contain any hole of outer (holes are empty space in outer)
            // Actually, inner's vertices being in outer is sufficient for containment
            // since inner is a simple polygon
            true
        }

        // Polygon contains MultiPolygon
        (Geometry::Polygon(_outer), Geometry::MultiPolygon(mp)) => {
            mp.polygons.iter().all(|poly| st_contains(container, &Geometry::Polygon(poly.clone())))
        }

        // MultiPolygon contains Polygon
        (Geometry::MultiPolygon(mp), Geometry::Polygon(_inner)) => {
            mp.polygons.iter().any(|poly| st_contains(&Geometry::Polygon(poly.clone()), contained))
        }

        // MultiPolygon contains Point
        (Geometry::MultiPolygon(mp), Geometry::Point(_pt)) => {
            mp.polygons.iter().any(|poly| st_contains(&Geometry::Polygon(poly.clone()), contained))
        }

        // MultiPolygon contains MultiPoint
        (Geometry::MultiPolygon(mp), Geometry::MultiPoint(mpoints)) => {
            mpoints.points.iter().all(|pt| {
                mp.polygons.iter().any(|poly| st_contains(&Geometry::Polygon(poly.clone()), &Geometry::Point(*pt)))
            })
        }

        // MultiPolygon contains LineString
        (Geometry::MultiPolygon(mp), Geometry::LineString(ls)) => {
            ls.points.iter().all(|pt| {
                mp.polygons.iter().any(|poly| st_contains(&Geometry::Polygon(poly.clone()), &Geometry::Point(*pt)))
            })
        }

        // MultiPolygon contains MultiPolygon
        (Geometry::MultiPolygon(_mp), Geometry::MultiPolygon(inner)) => {
            inner.polygons.iter().all(|poly| st_contains(container, &Geometry::Polygon(poly.clone())))
        }

        // LineString contains Point (on the line)
        (Geometry::LineString(ls), Geometry::Point(pt)) => {
            ls.points.iter().any(|p| p.x == pt.x && p.y == pt.y) ||
            is_point_on_linestring(pt, ls)
        }

        // LineString contains MultiPoint
        (Geometry::LineString(_ls), Geometry::MultiPoint(mp)) => {
            mp.points.iter().all(|pt| st_contains(container, &Geometry::Point(*pt)))
        }

        // Default: check using MBR containment (already verified above)
        _ => true,
    }
}

// ============================================================================
// ST_Intersects implementation
// ============================================================================

/// Returns true if the two geometries intersect.
///
/// Two geometries intersect if their MBRs intersect AND their interiors intersect.
/// This is the inverse of ST_Disjoint.
pub fn st_intersects(a: &Geometry, b: &Geometry) -> bool {
    // MBR pre-filter
    let mbr_a = a.mbr();
    let mbr_b = b.mbr();

    if !mbr_a.intersects(&mbr_b) {
        return false;
    }

    // If MBRs intersect, geometries might intersect
    match (a, b) {
        // Point-Point intersection
        (Geometry::Point(p1), Geometry::Point(p2)) => {
            p1.x == p2.x && p1.y == p2.y
        }

        // Point-LineString intersection
        (Geometry::Point(pt), Geometry::LineString(ls)) |
        (Geometry::LineString(ls), Geometry::Point(pt)) => {
            is_point_on_linestring(pt, ls)
        }

        // Point-Polygon intersection
        (Geometry::Point(pt), Geometry::Polygon(poly)) |
        (Geometry::Polygon(poly), Geometry::Point(pt)) => {
            point_in_polygon(pt, poly)
        }

        // Point-MultiPoint intersection
        (Geometry::Point(pt), Geometry::MultiPoint(mp)) |
        (Geometry::MultiPoint(mp), Geometry::Point(pt)) => {
            mp.points.iter().any(|p| p.x == pt.x && p.y == pt.y)
        }

        // LineString-LineString intersection
        (Geometry::LineString(ls1), Geometry::LineString(ls2)) => {
            linestrings_intersect(ls1, ls2)
        }

        // LineString-Polygon intersection
        (Geometry::LineString(ls), Geometry::Polygon(poly)) |
        (Geometry::Polygon(poly), Geometry::LineString(ls)) => {
            // Any vertex in polygon or any edge crossing polygon boundary
            for pt in &ls.points {
                if point_in_polygon(pt, poly) {
                    return true;
                }
            }
            // Check for edge crossings
            for edge in poly.edges() {
                for ls_edge in linestring_edges(ls) {
                    if segments_intersect(&edge.p1, &edge.p2, &ls_edge.p1, &ls_edge.p2) {
                        return true;
                    }
                }
            }
            false
        }

        // Polygon-Polygon intersection
        (Geometry::Polygon(p1), Geometry::Polygon(p2)) => {
            polygons_intersect(p1, p2)
        }

        // Default: MBR intersection is sufficient
        _ => true,
    }
}

/// Check if point lies on a linestring (including endpoints)
fn is_point_on_linestring(point: &Point, linestring: &LineString) -> bool {
    for edge in linestring_edges(linestring) {
        if point_on_segment(point, &edge.p1, &edge.p2) {
            return true;
        }
    }
    false
}

/// Get edges of a linestring
fn linestring_edges(ls: &LineString) -> Vec<Edge> {
    ls.points
        .windows(2)
        .map(|w| Edge::new(w[0], w[1]))
        .collect()
}

/// Check if two line segments intersect (including endpoints)
fn segments_intersect(p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> bool {
    let d1 = direction(p3, p4, p1);
    let d2 = direction(p3, p4, p2);
    let d3 = direction(p1, p2, p3);
    let d4 = direction(p1, p2, p4);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0)) &&
       ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0)) {
        return true;
    }

    if d1 == 0.0 && on_segment(p3, p4, p1) { return true; }
    if d2 == 0.0 && on_segment(p3, p4, p2) { return true; }
    if d3 == 0.0 && on_segment(p1, p2, p3) { return true; }
    if d4 == 0.0 && on_segment(p1, p2, p4) { return true; }

    false
}

/// Cross product magnitude for three points
fn direction(p1: &Point, p2: &Point, p3: &Point) -> f64 {
    (p3.x - p1.x) * (p2.y - p1.y) - (p2.x - p1.x) * (p3.y - p1.y)
}

/// Check if point q lies on segment pr
fn on_segment(p: &Point, q: &Point, r: &Point) -> bool {
    q.x <= p.x.max(r.x) && q.x >= p.x.min(r.x) &&
    q.y <= p.y.max(r.y) && q.y >= p.y.min(r.y)
}

/// Check if two linestrings intersect
fn linestrings_intersect(ls1: &LineString, ls2: &LineString) -> bool {
    for edge1 in linestring_edges(ls1) {
        for edge2 in linestring_edges(ls2) {
            if segments_intersect(&edge1.p1, &edge1.p2, &edge2.p1, &edge2.p2) {
                return true;
            }
        }
    }
    false
}

/// Check if two polygons intersect
fn polygons_intersect(p1: &Polygon, p2: &Polygon) -> bool {
    // Any vertex of p1 in p2 or vice versa
    for pt in &p1.exterior.points {
        if point_in_polygon(pt, p2) {
            return true;
        }
    }
    for pt in &p2.exterior.points {
        if point_in_polygon(pt, p1) {
            return true;
        }
    }

    // Any edges intersect
    for edge1 in p1.edges() {
        for edge2 in p2.edges() {
            if segments_intersect(&edge1.p1, &edge1.p2, &edge2.p1, &edge2.p2) {
                return true;
            }
        }
    }

    false
}

// ============================================================================
// ST_Within implementation
// ============================================================================

/// Returns true if the first geometry is within the second.
///
/// A.within(B) is equivalent to B.contains(A).
pub fn st_within(a: &Geometry, b: &Geometry) -> bool {
    st_contains(b, a)
}

// ============================================================================
// ST_Equals implementation
// ============================================================================

/// Returns true if the two geometries are spatially equal.
///
/// Two geometries are equal if they have the same MBR and the same geometric
/// structure (accounting for floating-point precision).
pub fn st_equals(a: &Geometry, b: &Geometry) -> bool {
    // MBR must match first
    let mbr_a = a.mbr();
    let mbr_b = b.mbr();

    if mbr_a != mbr_b {
        return false;
    }

    match (a, b) {
        (Geometry::Point(p1), Geometry::Point(p2)) => {
            points_equal(p1, p2)
        }

        (Geometry::LineString(ls1), Geometry::LineString(ls2)) => {
            linestrings_equal(ls1, ls2)
        }

        (Geometry::Polygon(poly1), Geometry::Polygon(poly2)) => {
            polygons_equal(poly1, poly2)
        }

        (Geometry::MultiPoint(mp1), Geometry::MultiPoint(mp2)) => {
            mp1.points.len() == mp2.points.len() &&
            mp1.points.iter().zip(mp2.points.iter()).all(|(p1, p2)| points_equal(p1, p2))
        }

        (Geometry::MultiLineString(mls1), Geometry::MultiLineString(mls2)) => {
            mls1.lines.len() == mls2.lines.len() &&
            mls1.lines.iter().zip(mls2.lines.iter()).all(|(ls1, ls2)| linestrings_equal(ls1, ls2))
        }

        (Geometry::MultiPolygon(mp1), Geometry::MultiPolygon(mp2)) => {
            mp1.polygons.len() == mp2.polygons.len() &&
            mp1.polygons.iter().zip(mp2.polygons.iter()).all(|(p1, p2)| polygons_equal(p1, p2))
        }

        _ => false,
    }
}

/// Check if two points are equal within floating-point tolerance
fn points_equal(p1: &Point, p2: &Point) -> bool {
    (p1.x - p2.x).abs() < f64::EPSILON && (p1.y - p2.y).abs() < f64::EPSILON
}

/// Check if two linestrings are equal
fn linestrings_equal(ls1: &LineString, ls2: &LineString) -> bool {
    if ls1.points.len() != ls2.points.len() {
        return false;
    }
    ls1.points.iter().zip(ls2.points.iter()).all(|(p1, p2)| points_equal(p1, p2))
}

/// Check if two polygons are equal
fn polygons_equal(p1: &Polygon, p2: &Polygon) -> bool {
    linestrings_equal(&p1.exterior, &p2.exterior) &&
    p1.holes.len() == p2.holes.len() &&
    p1.holes.iter().zip(p2.holes.iter()).all(|(h1, h2)| linestrings_equal(h1, h2))
}

// ============================================================================
// ST_Distance implementation
// ============================================================================

/// Returns the distance between two geometries.
///
/// For point-geometry, returns the minimum distance to any vertex.
/// For geometry-geometry, returns the minimum distance between any two points.
pub fn st_distance(a: &Geometry, b: &Geometry) -> f64 {
    match (a, b) {
        (Geometry::Point(p), Geometry::Point(q)) => {
            ((p.x - q.x).powi(2) + (p.y - q.y).powi(2)).sqrt()
        }

        (Geometry::Point(pt), Geometry::LineString(ls)) |
        (Geometry::LineString(ls), Geometry::Point(pt)) => {
            point_to_linestring_distance(pt, ls)
        }

        (Geometry::Point(pt), Geometry::Polygon(poly)) |
        (Geometry::Polygon(poly), Geometry::Point(pt)) => {
            point_to_polygon_distance(pt, poly)
        }

        (Geometry::LineString(ls1), Geometry::LineString(ls2)) => {
            linestring_to_linestring_distance(ls1, ls2)
        }

        (Geometry::LineString(ls), Geometry::Polygon(poly)) |
        (Geometry::Polygon(poly), Geometry::LineString(ls)) => {
            linestring_to_polygon_distance(ls, poly)
        }

        (Geometry::Polygon(p1), Geometry::Polygon(p2)) => {
            polygon_to_polygon_distance(p1, p2)
        }

        _ => {
            // Fallback to MBR distance for complex geometries
            let mbr_a = a.mbr();
            let mbr_b = b.mbr();
            mbr_distance(&mbr_a, &mbr_b)
        }
    }
}

/// Minimum distance from a point to a linestring
fn point_to_linestring_distance(point: &Point, linestring: &LineString) -> f64 {
    let mut min_dist = f64::INFINITY;
    for edge in linestring_edges(linestring) {
        let dist = point_to_segment_distance(point, &edge.p1, &edge.p2);
        min_dist = min_dist.min(dist);
    }
    min_dist
}

/// Minimum distance from a point to a polygon (exterior boundary)
fn point_to_polygon_distance(point: &Point, polygon: &Polygon) -> f64 {
    // If point is inside polygon, distance is 0
    if point_in_polygon(point, polygon) {
        return 0.0;
    }

    // Find minimum distance to any edge
    let mut min_dist = f64::INFINITY;
    for edge in polygon.edges() {
        let dist = point_to_segment_distance(point, &edge.p1, &edge.p2);
        min_dist = min_dist.min(dist);
    }
    min_dist
}

/// Minimum distance between two linestrings
fn linestring_to_linestring_distance(ls1: &LineString, ls2: &LineString) -> f64 {
    let mut min_dist = f64::INFINITY;
    for edge1 in linestring_edges(ls1) {
        for edge2 in linestring_edges(ls2) {
            let dist = segment_to_segment_distance(&edge1.p1, &edge1.p2, &edge2.p1, &edge2.p2);
            min_dist = min_dist.min(dist);
        }
    }
    min_dist
}

/// Minimum distance between a linestring and a polygon
fn linestring_to_polygon_distance(ls: &LineString, poly: &Polygon) -> f64 {
    let mut min_dist = f64::INFINITY;

    // Check if any vertex is inside polygon (distance = 0)
    for pt in &ls.points {
        if point_in_polygon(pt, poly) {
            return 0.0;
        }
    }

    // Minimum distance between edges
    for edge1 in linestring_edges(ls) {
        for edge2 in poly.edges() {
            let dist = segment_to_segment_distance(&edge1.p1, &edge1.p2, &edge2.p1, &edge2.p2);
            min_dist = min_dist.min(dist);
        }
    }
    min_dist
}

/// Minimum distance between two polygons
fn polygon_to_polygon_distance(p1: &Polygon, p2: &Polygon) -> f64 {
    // Check if polygons intersect (distance = 0)
    if polygons_intersect(p1, p2) {
        return 0.0;
    }

    // Check if one is inside the other
    for pt in &p1.exterior.points {
        if point_in_polygon(pt, p2) {
            return 0.0;
        }
    }
    for pt in &p2.exterior.points {
        if point_in_polygon(pt, p1) {
            return 0.0;
        }
    }

    // Minimum distance between edges
    let mut min_dist = f64::INFINITY;
    for edge1 in p1.edges() {
        for edge2 in p2.edges() {
            let dist = segment_to_segment_distance(&edge1.p1, &edge1.p2, &edge2.p1, &edge2.p2);
            min_dist = min_dist.min(dist);
        }
    }
    min_dist
}

/// Minimum distance from a point to a line segment
fn point_to_segment_distance(point: &Point, seg_start: &Point, seg_end: &Point) -> f64 {
    let Point { x: px, y: py } = *point;
    let (x1, y1) = (seg_start.x, seg_start.y);
    let (x2, y2) = (seg_end.x, seg_end.y);

    let dx = x2 - x1;
    let dy = y2 - y1;
    let length_sq = dx * dx + dy * dy;

    if length_sq < f64::EPSILON {
        // Segment is a point
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }

    // Project point onto line, clamped to segment
    let t = ((px - x1) * dx + (py - y1) * dy) / length_sq;
    let t_clamped = t.clamp(0.0, 1.0);

    let proj_x = x1 + t_clamped * dx;
    let proj_y = y1 + t_clamped * dy;

    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

/// Minimum distance between two line segments
fn segment_to_segment_distance(p1: &Point, p2: &Point, p3: &Point, p4: &Point) -> f64 {
    // Check if segments intersect
    if segments_intersect(p1, p2, p3, p4) {
        return 0.0;
    }

    // Minimum distance from endpoints
    let d1 = point_to_segment_distance(p1, p3, p4);
    let d2 = point_to_segment_distance(p2, p3, p4);
    let d3 = point_to_segment_distance(p3, p1, p2);
    let d4 = point_to_segment_distance(p4, p1, p2);

    d1.min(d2).min(d3).min(d4)
}

/// Distance between two MBRs
fn mbr_distance(a: &MBR, b: &MBR) -> f64 {
    let dx = if a.max_x < b.min_x {
        b.min_x - a.max_x
    } else if a.min_x > b.max_x {
        a.min_x - b.max_x
    } else {
        0.0
    };

    let dy = if a.max_y < b.min_y {
        b.min_y - a.max_y
    } else if a.min_y > b.max_y {
        a.min_y - b.max_y
    } else {
        0.0
    };

    (dx * dx + dy * dy).sqrt()
}

// ============================================================================
// MBR-based filtering with R-Tree
// ============================================================================

/// Filter geometries using MBR pre-filtering, then apply precise predicate
///
/// This is useful when you have an R-Tree index and want to efficiently
/// query for geometries that satisfy a spatial predicate.
///
/// # Arguments
///
/// * `candidates` - Iterator of (geometry_id, geometry) pairs from R-Tree search
/// * `query` - The query geometry
/// * `predicate` - The spatial predicate function to apply
///
/// # Example
///
/// ```
/// use sqlrustgo_gis::{Geometry, Point, Polygon, RTree, spatial::{st_contains, mbr_filter}};
/// use sqlrustgo_gis::rtree::MBRTrait;
///
/// let mut tree = RTree::new();
/// let square1 = Geometry::Polygon(Polygon::from_points(
///     vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0), Point::new(10.0, 10.0), Point::new(0.0, 10.0), Point::new(0.0, 0.0)],
///     vec![],
/// ).unwrap());
/// let square2 = Geometry::Polygon(Polygon::from_points(
///     vec![Point::new(5.0, 5.0), Point::new(15.0, 5.0), Point::new(15.0, 15.0), Point::new(5.0, 15.0), Point::new(5.0, 5.0)],
///     vec![],
/// ).unwrap());
///
/// tree.insert(&square1);
/// tree.insert(&square2);
///
/// let query_point = Geometry::Point(Point::new(7.0, 7.0));
///
/// // Get candidate IDs from R-Tree
/// let candidate_ids = tree.search_intersects(&query_point);
/// ```
pub fn mbr_filter<'a, I, F>(candidates: I, query: &Geometry, predicate: F) -> Vec<u64>
where
    I: Iterator<Item = (&'a u64, &'a Geometry)>,
    F: Fn(&'a Geometry, &Geometry) -> bool,
{
    let query_mbr = query.mbr();
    candidates
        .filter(|(_id, geom)| {
            // MBR pre-filter
            if !geom.mbr().intersects(&query_mbr) {
                return false;
            }
            // Precise predicate
            predicate(geom, query)
        })
        .map(|(&id, _)| id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a simple square polygon
    fn square_polygon(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Geometry {
        Geometry::Polygon(Polygon::from_points(
            vec![
                Point::new(min_x, min_y),
                Point::new(max_x, min_y),
                Point::new(max_x, max_y),
                Point::new(min_x, max_y),
                Point::new(min_x, min_y),
            ],
            vec![],
        ).unwrap())
    }

    // Helper to create a point
    fn point(x: f64, y: f64) -> Geometry {
        Geometry::Point(Point::new(x, y))
    }

    #[test]
    fn test_point_in_polygon_simple() {
        let poly = Polygon::from_points(
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(10.0, 10.0),
                Point::new(0.0, 10.0),
                Point::new(0.0, 0.0),
            ],
            vec![],
        ).unwrap();

        // Inside
        assert!(point_in_polygon(&Point::new(5.0, 5.0), &poly));

        // Outside
        assert!(!point_in_polygon(&Point::new(15.0, 15.0), &poly));

        // On edge (not inside with ray casting)
        assert!(!point_in_polygon(&Point::new(5.0, 0.0), &poly));
    }

    #[test]
    fn test_point_in_polygon_complex() {
        // L-shaped polygon: bottom bar (x: 0-10, y: 0-5) + left bar (x: 0-5, y: 5-10)
        // The notch is x: 5-10, y: 5-10 (a 5x5 square)
        let poly = Polygon::from_points(
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(10.0, 5.0),
                Point::new(5.0, 5.0),
                Point::new(5.0, 10.0),
                Point::new(0.0, 10.0),
                Point::new(0.0, 0.0),
            ],
            vec![],
        ).unwrap();

        // Inside L-shape (bottom bar and left bar)
        assert!(point_in_polygon(&Point::new(2.0, 2.0), &poly)); // in bottom bar
        assert!(point_in_polygon(&Point::new(2.0, 7.0), &poly)); // in left bar
        assert!(point_in_polygon(&Point::new(7.0, 2.0), &poly)); // in bottom bar (x=7 is within 0-10)

        // Outside L-shape (in the notch at x: 5-10, y: 5-10)
        assert!(!point_in_polygon(&Point::new(7.0, 7.0), &poly)); // in the notch

        // Outside L-shape (to the right of bottom bar)
        assert!(!point_in_polygon(&Point::new(12.0, 2.0), &poly));
    }

    #[test]
    fn test_point_on_segment() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 0.0);

        // Point on segment
        assert!(point_on_segment(&Point::new(5.0, 0.0), &p1, &p2));

        // Point at endpoint
        assert!(point_on_segment(&Point::new(0.0, 0.0), &p1, &p2));
        assert!(point_on_segment(&Point::new(10.0, 0.0), &p1, &p2));

        // Point not on segment
        assert!(!point_on_segment(&Point::new(5.0, 1.0), &p1, &p2));
        assert!(!point_on_segment(&Point::new(15.0, 0.0), &p1, &p2));
    }

    #[test]
    fn test_st_contains_point_polygon() {
        let square = square_polygon(0.0, 10.0, 0.0, 10.0);

        // Point inside
        assert!(st_contains(&square, &point(5.0, 5.0)));

        // Point outside
        assert!(!st_contains(&square, &point(15.0, 15.0)));

        // Point on edge (contained)
        assert!(st_contains(&square, &point(5.0, 0.0)));
    }

    #[test]
    fn test_st_contains_polygon_polygon() {
        let outer = square_polygon(0.0, 10.0, 0.0, 10.0);
        let inner = square_polygon(2.0, 8.0, 2.0, 8.0);
        let disjoint = square_polygon(20.0, 30.0, 20.0, 30.0);

        // Inner polygon inside outer
        assert!(st_contains(&outer, &inner));

        // Outer does not contain disjoint
        assert!(!st_contains(&outer, &disjoint));

        // Disjoint does not contain outer
        assert!(!st_contains(&disjoint, &outer));
    }

    #[test]
    fn test_st_intersects_point_point() {
        // Same point
        assert!(st_intersects(&point(5.0, 5.0), &point(5.0, 5.0)));

        // Different points
        assert!(!st_intersects(&point(5.0, 5.0), &point(10.0, 10.0)));
    }

    #[test]
    fn test_st_intersects_point_polygon() {
        let square = square_polygon(0.0, 10.0, 0.0, 10.0);

        // Point inside
        assert!(st_intersects(&square, &point(5.0, 5.0)));

        // Point outside
        assert!(!st_intersects(&square, &point(15.0, 15.0)));
    }

    #[test]
    fn test_st_intersects_polygons() {
        let square1 = square_polygon(0.0, 10.0, 0.0, 10.0);
        let square2 = square_polygon(5.0, 15.0, 5.0, 15.0); // Overlapping
        let disjoint = square_polygon(20.0, 30.0, 20.0, 30.0); // No overlap

        // Overlapping polygons intersect
        assert!(st_intersects(&square1, &square2));

        // Disjoint polygons do not intersect
        assert!(!st_intersects(&square1, &disjoint));
    }

    #[test]
    fn test_st_intersects_linestring_polygon() {
        let square = square_polygon(0.0, 10.0, 0.0, 10.0);

        // Line completely inside
        let line_inside = Geometry::LineString(
            LineString::new(vec![
                Point::new(2.0, 2.0),
                Point::new(5.0, 5.0),
                Point::new(8.0, 8.0),
            ]).unwrap()
        );
        assert!(st_intersects(&square, &line_inside));

        // Line crossing boundary
        let line_crossing = Geometry::LineString(
            LineString::new(vec![
                Point::new(-5.0, 5.0),
                Point::new(5.0, 5.0),
            ]).unwrap()
        );
        assert!(st_intersects(&square, &line_crossing));

        // Line completely outside
        let line_outside = Geometry::LineString(
            LineString::new(vec![
                Point::new(15.0, 15.0),
                Point::new(20.0, 20.0),
            ]).unwrap()
        );
        assert!(!st_intersects(&square, &line_outside));
    }

    #[test]
    fn test_st_within() {
        let outer = square_polygon(0.0, 10.0, 0.0, 10.0);
        let inner = square_polygon(2.0, 8.0, 2.0, 8.0);

        // inner is within outer
        assert!(st_within(&inner, &outer));

        // outer is not within inner
        assert!(!st_within(&outer, &inner));

        // Same geometry is within itself (for polygons this works)
        assert!(st_within(&inner, &inner));
    }

    #[test]
    fn test_st_equals() {
        let poly1 = square_polygon(0.0, 10.0, 0.0, 10.0);
        let poly2 = square_polygon(0.0, 10.0, 0.0, 10.0);
        let different = square_polygon(0.0, 10.0, 0.0, 20.0);

        // Same polygon
        assert!(st_equals(&poly1, &poly2));

        // Different polygon
        assert!(!st_equals(&poly1, &different));

        // Same point
        assert!(st_equals(&point(5.0, 5.0), &point(5.0, 5.0)));

        // Different point
        assert!(!st_equals(&point(5.0, 5.0), &point(5.0, 6.0)));
    }

    #[test]
    fn test_st_distance_point_point() {
        let p1 = point(0.0, 0.0);
        let p2 = point(3.0, 4.0);

        // 3-4-5 triangle
        assert!((st_distance(&p1, &p2) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_st_distance_point_polygon() {
        let square = square_polygon(0.0, 10.0, 0.0, 10.0);

        // Point inside - distance 0
        assert!((st_distance(&square, &point(5.0, 5.0)) - 0.0).abs() < f64::EPSILON);

        // Point outside - distance to nearest edge
        // Point at (15, 5) is 5 units from edge at x=10
        let dist = st_distance(&square, &point(15.0, 5.0));
        assert!((dist - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_st_distance_polygon_polygon() {
        let square1 = square_polygon(0.0, 10.0, 0.0, 10.0);
        let square2 = square_polygon(20.0, 30.0, 20.0, 30.0); // Disjoint

        // Distance between disjoint squares [0,10]x[0,10] and [20,30]x[20,30]
        // Nearest points: (10,10) and (20,20), distance = sqrt((20-10)^2 + (20-10)^2) = sqrt(200) ≈ 14.14
        let dist = st_distance(&square1, &square2);
        let expected = (200.0_f64).sqrt();
        assert!((dist - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_st_distance_overlapping() {
        let square1 = square_polygon(0.0, 10.0, 0.0, 10.0);
        let square2 = square_polygon(5.0, 15.0, 5.0, 15.0); // Overlapping

        // Overlapping polygons have distance 0
        let dist = st_distance(&square1, &square2);
        assert!((dist - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_polygon_edges() {
        let poly = Polygon::from_points(
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(10.0, 10.0),
                Point::new(0.0, 10.0),
                Point::new(0.0, 0.0),
            ],
            vec![],
        ).unwrap();

        let edges: Vec<_> = poly.edges().collect();
        assert_eq!(edges.len(), 4);

        // Check first edge
        assert_eq!(edges[0].p1.x, 0.0);
        assert_eq!(edges[0].p1.y, 0.0);
        assert_eq!(edges[0].p2.x, 10.0);
        assert_eq!(edges[0].p2.y, 0.0);
    }

    #[test]
    fn test_segments_intersect() {
        // Horizontal and vertical crossing
        let p1 = Point::new(0.0, 5.0);
        let p2 = Point::new(10.0, 5.0);
        let p3 = Point::new(5.0, 0.0);
        let p4 = Point::new(5.0, 10.0);

        assert!(segments_intersect(&p1, &p2, &p3, &p4));

        // Non-intersecting
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(5.0, 5.0);
        let p3 = Point::new(10.0, 0.0);
        let p4 = Point::new(10.0, 5.0);

        assert!(!segments_intersect(&p1, &p2, &p3, &p4));
    }

    #[test]
    fn test_segments_intersect_at_endpoint() {
        // Segments that share an endpoint
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(5.0, 5.0);
        let p3 = Point::new(5.0, 5.0);
        let p4 = Point::new(10.0, 0.0);

        // They intersect at the shared endpoint
        assert!(segments_intersect(&p1, &p2, &p3, &p4));
    }

    #[test]
    fn test_linestring_edges() {
        let ls = LineString::new(vec![
            Point::new(0.0, 0.0),
            Point::new(5.0, 5.0),
            Point::new(10.0, 0.0),
        ]).unwrap();

        let edges = linestring_edges(&ls);
        assert_eq!(edges.len(), 2);

        assert_eq!(edges[0].p1, Point::new(0.0, 0.0));
        assert_eq!(edges[0].p2, Point::new(5.0, 5.0));
        assert_eq!(edges[1].p1, Point::new(5.0, 5.0));
        assert_eq!(edges[1].p2, Point::new(10.0, 0.0));
    }

    #[test]
    fn test_st_contains_multipoint() {
        let square = square_polygon(0.0, 10.0, 0.0, 10.0);

        let multipoint = Geometry::MultiPoint(
            MultiPoint::new(vec![
                Point::new(2.0, 2.0),
                Point::new(5.0, 5.0),
                Point::new(8.0, 8.0),
            ]).unwrap()
        );

        // All points inside
        assert!(st_contains(&square, &multipoint));

        // One point outside
        let multipoint_partial = Geometry::MultiPoint(
            MultiPoint::new(vec![
                Point::new(2.0, 2.0),
                Point::new(15.0, 15.0), // Outside
            ]).unwrap()
        );

        assert!(!st_contains(&square, &multipoint_partial));
    }

    #[test]
    fn test_st_contains_linestring() {
        let square = square_polygon(0.0, 10.0, 0.0, 10.0);

        // Line completely inside
        let line_inside = Geometry::LineString(
            LineString::new(vec![
                Point::new(2.0, 2.0),
                Point::new(8.0, 8.0),
            ]).unwrap()
        );
        assert!(st_contains(&square, &line_inside));

        // Line extending outside
        let line_outside = Geometry::LineString(
            LineString::new(vec![
                Point::new(2.0, 2.0),
                Point::new(15.0, 15.0), // Outside
            ]).unwrap()
        );
        assert!(!st_contains(&square, &line_outside));
    }

    #[test]
    fn test_point_to_segment_distance() {
        let seg_start = Point::new(0.0, 0.0);
        let seg_end = Point::new(10.0, 0.0);

        // Point perpendicular to segment
        let pt = Point::new(5.0, 5.0);
        assert!((point_to_segment_distance(&pt, &seg_start, &seg_end) - 5.0).abs() < f64::EPSILON);

        // Point at endpoint
        let pt = Point::new(0.0, 0.0);
        assert!((point_to_segment_distance(&pt, &seg_start, &seg_end) - 0.0).abs() < f64::EPSILON);

        // Point beyond endpoint
        let pt = Point::new(15.0, 0.0);
        assert!((point_to_segment_distance(&pt, &seg_start, &seg_end) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mbr_contains_and_intersects() {
        let mbr1 = MBR::new(0.0, 10.0, 0.0, 10.0);
        let mbr2 = MBR::new(2.0, 8.0, 2.0, 8.0); // Inside mbr1
        let mbr3 = MBR::new(5.0, 15.0, 5.0, 15.0); // Overlaps mbr1
        let mbr4 = MBR::new(20.0, 30.0, 20.0, 30.0); // Disjoint

        // mbr1 contains mbr2
        assert!(mbr_contains(&mbr1, &mbr2));
        assert!(!mbr_contains(&mbr2, &mbr1));

        // mbr1 intersects mbr3
        assert!(mbr_intersects(&mbr1, &mbr3));

        // mbr1 does not intersect mbr4
        assert!(!mbr_intersects(&mbr1, &mbr4));
    }

    #[test]
    fn test_multipolygon_contains() {
        let mp = Geometry::MultiPolygon(MultiPolygon::new(vec![
            Polygon::from_points(
                vec![
                    Point::new(0.0, 0.0),
                    Point::new(10.0, 0.0),
                    Point::new(10.0, 10.0),
                    Point::new(0.0, 10.0),
                    Point::new(0.0, 0.0),
                ],
                vec![],
            ).unwrap(),
            Polygon::from_points(
                vec![
                    Point::new(20.0, 20.0),
                    Point::new(30.0, 20.0),
                    Point::new(30.0, 30.0),
                    Point::new(20.0, 30.0),
                    Point::new(20.0, 20.0),
                ],
                vec![],
            ).unwrap(),
        ]).unwrap());

        // Point inside first polygon
        assert!(st_contains(&mp, &point(5.0, 5.0)));

        // Point inside second polygon
        assert!(st_contains(&mp, &point(25.0, 25.0)));

        // Point outside both
        assert!(!st_contains(&mp, &point(15.0, 15.0)));
    }

    #[test]
    fn test_polygon_with_hole_contains() {
        // Square with a hole (inner square from 4 to 6)
        let exterior = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
            Point::new(0.0, 0.0),
        ];
        let hole = vec![
            Point::new(4.0, 4.0),
            Point::new(6.0, 4.0),
            Point::new(6.0, 6.0),
            Point::new(4.0, 6.0),
            Point::new(4.0, 4.0),
        ];

        let poly_with_hole = Geometry::Polygon(
            Polygon::from_points(exterior, vec![hole]).unwrap()
        );

        // Point in the "doughnut" area (inside outer, outside hole)
        assert!(st_contains(&poly_with_hole, &point(2.0, 2.0)));

        // Point in the hole (not contained)
        assert!(!st_contains(&poly_with_hole, &point(5.0, 5.0)));

        // Point outside entirely
        assert!(!st_contains(&poly_with_hole, &point(15.0, 15.0)));
    }
}