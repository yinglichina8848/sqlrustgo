//! R-Tree spatial index implementation for GIS data
//!
//! Provides efficient spatial queries using R-Tree data structure with
//! quadratic splitting algorithm.

use std::fmt;

use crate::geometry::{Geometry, Point};

/// Minimum Bounding Rectangle for spatial indexing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MBR {
    /// Minimum x coordinate
    pub min_x: f64,
    /// Maximum x coordinate
    pub max_x: f64,
    /// Minimum y coordinate
    pub min_y: f64,
    /// Maximum y coordinate
    pub max_y: f64,
}

impl MBR {
    /// Create a new MBR from min/max coordinates
    pub fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        MBR {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Create an MBR from a single point (zero-area bounding box)
    pub fn from_point(p: &Point) -> Self {
        MBR {
            min_x: p.x,
            max_x: p.x,
            min_y: p.y,
            max_y: p.y,
        }
    }

    /// Create an infinite MBR that contains everything
    pub fn infinite() -> Self {
        MBR {
            min_x: f64::NEG_INFINITY,
            max_x: f64::INFINITY,
            min_y: f64::NEG_INFINITY,
            max_y: f64::INFINITY,
        }
    }

    /// Check if this MBR is valid (min <= max for both dimensions)
    pub fn is_valid(&self) -> bool {
        self.min_x <= self.max_x && self.min_y <= self.max_y
    }

    /// Compute the area of this MBR
    pub fn area(&self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        (self.max_x - self.min_x) * (self.max_y - self.min_y)
    }

    /// Compute the perimeter of this MBR
    pub fn perimeter(&self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        2.0 * (self.max_x - self.min_x + self.max_y - self.min_y)
    }

    /// Compute the enlarged area when combining this MBR with another
    pub fn union_area(&self, other: &MBR) -> f64 {
        let min_x = self.min_x.min(other.min_x);
        let max_x = self.max_x.max(other.max_x);
        let min_y = self.min_y.min(other.min_y);
        let max_y = self.max_y.max(other.max_y);
        (max_x - min_x) * (max_y - min_y)
    }

    /// Check if this MBR intersects with another MBR
    pub fn intersects(&self, other: &MBR) -> bool {
        !(self.max_x < other.min_x
            || self.min_x > other.max_x
            || self.max_y < other.min_y
            || self.min_y > other.max_y)
    }

    /// Check if this MBR completely contains another MBR
    pub fn contains(&self, other: &MBR) -> bool {
        self.min_x <= other.min_x
            && self.max_x >= other.max_x
            && self.min_y <= other.min_y
            && self.max_y >= other.max_y
    }

    /// Check if this MBR contains a point
    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }

    /// Expand this MBR to include another MBR, modifying self
    pub fn expand(&mut self, other: &MBR) {
        self.min_x = self.min_x.min(other.min_x);
        self.max_x = self.max_x.max(other.max_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_y = self.max_y.max(other.max_y);
    }

    /// Compute the margin of this MBR (sum of sides)
    pub fn margin(&self) -> f64 {
        2.0 * (self.max_x - self.min_x + self.max_y - self.min_y)
    }

    /// Compute the overlap between this MBR and another
    pub fn overlap(&self, other: &MBR) -> f64 {
        if !self.intersects(other) {
            return 0.0;
        }
        let width = (self.max_x.min(other.max_x) - self.min_x.max(other.min_x)).max(0.0);
        let height = (self.max_y.min(other.max_y) - self.min_y.max(other.min_y)).max(0.0);
        width * height
    }

    /// Compute the distance from a point to the nearest edge of this MBR
    pub fn distance_to_point(&self, point: &Point) -> f64 {
        let dx = if point.x < self.min_x {
            self.min_x - point.x
        } else if point.x > self.max_x {
            point.x - self.max_x
        } else {
            0.0
        };
        let dy = if point.y < self.min_y {
            self.min_y - point.y
        } else if point.y > self.max_y {
            point.y - self.max_y
        } else {
            0.0
        };
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for MBR {
    fn default() -> Self {
        MBR::infinite()
    }
}

impl fmt::Display for MBR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MBR[({}, {}) - ({}, {})]",
            self.min_x, self.min_y, self.max_x, self.max_y
        )
    }
}

/// Entry in the R-Tree: a bounding box with an associated geometry ID
#[derive(Debug, Clone)]
pub struct Entry {
    /// The minimum bounding rectangle of this entry
    pub mbr: MBR,
    /// The ID of the geometry this entry represents
    pub geometry_id: u64,
}

impl Entry {
    /// Create a new entry with the given MBR and geometry ID
    pub fn new(mbr: MBR, geometry_id: u64) -> Self {
        Entry { mbr, geometry_id }
    }
}

/// R-Tree node variants
#[derive(Debug, Clone)]
pub enum RTreeNode {
    /// Leaf node containing entries
    Leaf {
        /// Entries in this leaf node (max 4 by default)
        entries: Vec<Entry>,
    },
    /// Internal node containing child nodes
    Internal {
        /// Child nodes
        children: Vec<RTreeNode>,
        /// Combined MBR of all children
        mbr: MBR,
    },
}

impl RTreeNode {
    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        matches!(self, RTreeNode::Leaf { .. })
    }

    /// Get the MBR of this node
    pub fn mbr(&self) -> MBR {
        match self {
            RTreeNode::Leaf { entries } => {
                if entries.is_empty() {
                    MBR::infinite()
                } else {
                    let mut mbr = entries[0].mbr;
                    for entry in entries.iter().skip(1) {
                        mbr.expand(&entry.mbr);
                    }
                    mbr
                }
            }
            RTreeNode::Internal { mbr, .. } => *mbr,
        }
    }

    /// Get the number of entries or children in this node
    pub fn len(&self) -> usize {
        match self {
            RTreeNode::Leaf { entries } => entries.len(),
            RTreeNode::Internal { children, .. } => children.len(),
        }
    }

    /// Check if this node is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Recompute and update the MBR of this node based on children
    pub fn update_mbr(&mut self) {
        if let RTreeNode::Internal { children, mbr } = self {
            *mbr = children.iter().map(|c| c.mbr()).fold(
                MBR::new(
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ),
                |acc, child_mbr| {
                    MBR::new(
                        acc.min_x.min(child_mbr.min_x),
                        acc.max_x.max(child_mbr.max_x),
                        acc.min_y.min(child_mbr.min_y),
                        acc.max_y.max(child_mbr.max_y),
                    )
                },
            );
        }
    }

    /// Flatten all entries from this node and its descendants
    pub fn flatten_entries(&self) -> Vec<Entry> {
        let mut entries = Vec::new();
        self.collect_entries(&mut entries);
        entries
    }

    fn collect_entries(&self, result: &mut Vec<Entry>) {
        match self {
            RTreeNode::Leaf { entries } => {
                result.extend(entries.clone());
            }
            RTreeNode::Internal { children, .. } => {
                for child in children {
                    child.collect_entries(result);
                }
            }
        }
    }

    /// Insert entry into this node recursively, returns true if node needs split
    fn insert(&mut self, entry: Entry, max_entries: usize) -> bool {
        match self {
            RTreeNode::Leaf { entries } => {
                entries.push(entry);
                entries.len() > max_entries
            }
            RTreeNode::Internal { children, .. } => {
                let search_mbr = entry.mbr;
                let mut best_idx = 0;
                let mut best_enlargement = f64::INFINITY;
                let mut best_area = f64::INFINITY;

                for (i, child) in children.iter().enumerate() {
                    let child_mbr = child.mbr();
                    let current_area = child_mbr.area();
                    let enlarged = child_mbr.union_area(&search_mbr);
                    let enlargement = enlarged - current_area;

                    if enlargement < best_enlargement
                        || (enlargement == best_enlargement && current_area < best_area)
                    {
                        best_enlargement = enlargement;
                        best_area = current_area;
                        best_idx = i;
                    }
                }

                let needs_split = children[best_idx].insert(entry, max_entries);
                self.update_mbr();
                needs_split
            }
        }
    }

    /// Remove entry by geometry ID, returns true if found and removed
    fn remove(&mut self, geometry_id: u64) -> bool {
        match self {
            RTreeNode::Leaf { entries } => {
                if let Some(pos) = entries.iter().position(|e| e.geometry_id == geometry_id) {
                    entries.remove(pos);
                    true
                } else {
                    false
                }
            }
            RTreeNode::Internal { children, .. } => {
                for child in children.iter_mut() {
                    if child.remove(geometry_id) {
                        self.update_mbr();
                        return true;
                    }
                }
                false
            }
        }
    }
}

/// R-Tree spatial index
///
/// Provides efficient spatial queries for geometry data using the R-Tree
/// data structure with quadratic splitting on node overflow.
#[derive(Debug, Clone)]
pub struct RTree {
    /// Root node of the R-Tree
    root: RTreeNode,
    /// Maximum entries per leaf node
    max_entries: usize,
    /// Minimum entries per leaf node (for underflow handling)
    min_entries: usize,
    /// Counter for generating geometry IDs
    next_geometry_id: u64,
}

impl Default for RTree {
    fn default() -> Self {
        RTree::new()
    }
}

impl RTree {
    /// Create a new R-Tree with default parameters
    pub fn new() -> Self {
        RTree {
            root: RTreeNode::Leaf {
                entries: Vec::new(),
            },
            max_entries: 4,
            min_entries: 2,
            next_geometry_id: 1,
        }
    }

    /// Create a new R-Tree with custom parameters
    pub fn with_params(max_entries: usize, min_entries: usize) -> Self {
        let min_entries = min_entries.min(max_entries / 2);
        RTree {
            root: RTreeNode::Leaf {
                entries: Vec::new(),
            },
            max_entries,
            min_entries,
            next_geometry_id: 1,
        }
    }

    /// Generate the next geometry ID
    fn next_id(&mut self) -> u64 {
        let id = self.next_geometry_id;
        self.next_geometry_id += 1;
        id
    }

    /// Insert a geometry and return its assigned ID
    pub fn insert(&mut self, geometry: &Geometry) -> u64 {
        let geometry_id = self.next_id();
        let mbr = geometry.mbr();
        let entry = Entry::new(mbr, geometry_id);
        self.insert_entry(entry);
        geometry_id
    }

    /// Insert an entry directly with a specific geometry ID
    pub fn insert_with_id(&mut self, geometry_id: u64, geometry: &Geometry) {
        let mbr = geometry.mbr();
        let entry = Entry::new(mbr, geometry_id);
        self.insert_entry(entry);
    }

    /// Insert an entry into the R-Tree
    fn insert_entry(&mut self, entry: Entry) {
        let needs_split = self.root.insert(entry, self.max_entries);
        if needs_split {
            self.split_root();
        }
    }

    /// Split the root node using quadratic split
    fn split_root(&mut self) {
        let entries = self.root.flatten_entries();
        let max_entries = self.max_entries;

        if entries.len() <= max_entries {
            self.root = RTreeNode::Leaf { entries };
            return;
        }

        let (group1, group2) = Self::quadratic_split(&entries, max_entries);

        self.root = RTreeNode::Internal {
            children: vec![
                RTreeNode::Leaf { entries: group1 },
                RTreeNode::Leaf { entries: group2 },
            ],
            mbr: MBR::infinite(),
        };

        self.root.update_mbr();
    }

    /// Quadratic split algorithm
    fn quadratic_split(entries: &[Entry], max_entries: usize) -> (Vec<Entry>, Vec<Entry>) {
        if entries.len() <= max_entries {
            return (entries.to_vec(), Vec::new());
        }

        let (seed1_idx, seed2_idx) = Self::pick_seeds(entries);

        let mut group1 = vec![entries[seed1_idx].clone()];
        let mut group2 = vec![entries[seed2_idx].clone()];

        let mbr1 = group1[0].mbr;
        let mbr2 = group2[0].mbr;

        let mut remaining: Vec<Entry> = entries
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != seed1_idx && *i != seed2_idx)
            .map(|(_, e)| e.clone())
            .collect();

        while !remaining.is_empty() {
            let (best_idx, _) = Self::pick_next(&remaining, &mbr1, &mbr2);

            let entry = remaining.remove(best_idx);

            let enlarged1 = mbr1.union_area(&entry.mbr);
            let enlarged2 = mbr2.union_area(&entry.mbr);
            let diff1 = enlarged1 - mbr1.area();
            let diff2 = enlarged2 - mbr2.area();

            if diff1 < diff2 {
                group1.push(entry);
            } else {
                group2.push(entry);
            }
        }

        (group1, group2)
    }

    /// Pick two seed entries for quadratic split
    fn pick_seeds(entries: &[Entry]) -> (usize, usize) {
        let mut max_waste = f64::NEG_INFINITY;
        let mut best_i = 0;
        let mut best_j = 1;

        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                let combined = entries[i].mbr.union_area(&entries[j].mbr);
                let waste = combined - entries[i].mbr.area() - entries[j].mbr.area();
                if waste > max_waste {
                    max_waste = waste;
                    best_i = i;
                    best_j = j;
                }
            }
        }

        (best_i, best_j)
    }

    /// Pick the next entry to assign based on minimum enlargement difference
    fn pick_next(entries: &[Entry], mbr1: &MBR, mbr2: &MBR) -> (usize, f64) {
        let mut best_idx = 0;
        let mut best_diff = f64::INFINITY;

        for (i, entry) in entries.iter().enumerate() {
            let area1 = mbr1.union_area(&entry.mbr) - mbr1.area();
            let area2 = mbr2.union_area(&entry.mbr) - mbr2.area();
            let diff = (area1 - area2).abs();

            if diff < best_diff {
                best_diff = diff;
                best_idx = i;
            }
        }

        (best_idx, best_diff)
    }

    /// Search for all entries that intersect with the given MBR
    pub fn search(&self, mbr: &MBR) -> Vec<u64> {
        let mut results = Vec::new();
        self.search_node(&self.root, mbr, &mut results);
        results
    }

    /// Recursively search for entries intersecting with the given MBR
    fn search_node(&self, node: &RTreeNode, search_mbr: &MBR, results: &mut Vec<u64>) {
        match node {
            RTreeNode::Leaf { entries } => {
                for entry in entries {
                    if search_mbr.intersects(&entry.mbr) {
                        results.push(entry.geometry_id);
                    }
                }
            }
            RTreeNode::Internal { children, .. } => {
                for child in children {
                    if search_mbr.intersects(&child.mbr()) {
                        self.search_node(child, search_mbr, results);
                    }
                }
            }
        }
    }

    /// Search for all geometries that contain the given point
    pub fn search_contains(&self, point: &Point) -> Vec<u64> {
        let mut results = Vec::new();
        self.search_contains_node(&self.root, point, &mut results);
        results
    }

    /// Recursively search for entries containing the point
    fn search_contains_node(&self, node: &RTreeNode, point: &Point, results: &mut Vec<u64>) {
        match node {
            RTreeNode::Leaf { entries } => {
                for entry in entries {
                    if entry.mbr.contains_point(point) {
                        results.push(entry.geometry_id);
                    }
                }
            }
            RTreeNode::Internal { children, mbr } => {
                if mbr.contains_point(point) {
                    for child in children {
                        self.search_contains_node(child, point, results);
                    }
                }
            }
        }
    }

    /// Search for all geometries that intersect with the given geometry
    pub fn search_intersects(&self, geometry: &Geometry) -> Vec<u64> {
        let mbr = geometry.mbr();
        self.search(&mbr)
    }

    /// Remove a geometry by its ID
    pub fn remove(&mut self, geometry_id: u64) -> bool {
        let removed = self.root.remove(geometry_id);
        if removed {
            self.handle_underflow();
        }
        removed
    }

    /// Handle underflow by merging or reinserting
    fn handle_underflow(&mut self) {
        let needs_rebuild = match &mut self.root {
            RTreeNode::Leaf { entries } => entries.is_empty(),
            RTreeNode::Internal { children, .. } => children.len() < self.min_entries,
        };

        if !needs_rebuild {
            return;
        }

        let entries = self.root.flatten_entries();
        let max_entries = self.max_entries;

        if entries.is_empty() {
            self.root = RTreeNode::Leaf {
                entries: Vec::new(),
            };
        } else if entries.len() <= max_entries {
            self.root = RTreeNode::Leaf { entries };
        } else {
            let (group1, group2) = Self::quadratic_split(&entries, max_entries);
            self.root = RTreeNode::Internal {
                children: vec![
                    RTreeNode::Leaf { entries: group1 },
                    RTreeNode::Leaf { entries: group2 },
                ],
                mbr: MBR::infinite(),
            };
            self.root.update_mbr();
        }
    }

    /// Get the number of geometries in the index
    pub fn len(&self) -> usize {
        self.root.flatten_entries().len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all geometry IDs in the index
    pub fn geometry_ids(&self) -> Vec<u64> {
        let entries = self.root.flatten_entries();
        entries.into_iter().map(|e| e.geometry_id).collect()
    }
}

/// Trait for computing MBR of a geometry
pub trait MBRTrait {
    /// Compute the minimum bounding rectangle of this geometry
    fn mbr(&self) -> MBR;
}

impl MBRTrait for Geometry {
    fn mbr(&self) -> MBR {
        match self {
            Geometry::Point(p) => MBR::from_point(p),
            Geometry::LineString(ls) => {
                if ls.points.is_empty() {
                    return MBR::infinite();
                }
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;

                for p in &ls.points {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                    min_y = min_y.min(p.y);
                    max_y = max_y.max(p.y);
                }

                MBR::new(min_x, max_x, min_y, max_y)
            }
            Geometry::Polygon(p) => {
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;

                for p in &p.exterior.points {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                    min_y = min_y.min(p.y);
                    max_y = max_y.max(p.y);
                }
                for hole in &p.holes {
                    for p in &hole.points {
                        min_x = min_x.min(p.x);
                        max_x = max_x.max(p.x);
                        min_y = min_y.min(p.y);
                        max_y = max_y.max(p.y);
                    }
                }

                MBR::new(min_x, max_x, min_y, max_y)
            }
            Geometry::MultiPoint(mp) => {
                if mp.points.is_empty() {
                    return MBR::infinite();
                }
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;

                for p in &mp.points {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                    min_y = min_y.min(p.y);
                    max_y = max_y.max(p.y);
                }

                MBR::new(min_x, max_x, min_y, max_y)
            }
            Geometry::MultiLineString(mls) => {
                if mls.lines.is_empty() {
                    return MBR::infinite();
                }
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;

                for ls in &mls.lines {
                    for p in &ls.points {
                        min_x = min_x.min(p.x);
                        max_x = max_x.max(p.x);
                        min_y = min_y.min(p.y);
                        max_y = max_y.max(p.y);
                    }
                }

                MBR::new(min_x, max_x, min_y, max_y)
            }
            Geometry::MultiPolygon(mp) => {
                if mp.polygons.is_empty() {
                    return MBR::infinite();
                }
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;

                for poly in &mp.polygons {
                    for p in &poly.exterior.points {
                        min_x = min_x.min(p.x);
                        max_x = max_x.max(p.x);
                        min_y = min_y.min(p.y);
                        max_y = max_y.max(p.y);
                    }
                    for hole in &poly.holes {
                        for p in &hole.points {
                            min_x = min_x.min(p.x);
                            max_x = max_x.max(p.x);
                            min_y = min_y.min(p.y);
                            max_y = max_y.max(p.y);
                        }
                    }
                }

                MBR::new(min_x, max_x, min_y, max_y)
            }
            Geometry::GeometryCollection(gc) => {
                if gc.geometries.is_empty() {
                    return MBR::infinite();
                }
                let mut min_x = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;

                for geo in &gc.geometries {
                    let geo_mbr = geo.mbr();
                    min_x = min_x.min(geo_mbr.min_x);
                    max_x = max_x.max(geo_mbr.max_x);
                    min_y = min_y.min(geo_mbr.min_y);
                    max_y = max_y.max(geo_mbr.max_y);
                }

                MBR::new(min_x, max_x, min_y, max_y)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{LineString, Point, Polygon};

    fn create_point(x: f64, y: f64) -> Geometry {
        Geometry::Point(Point::new(x, y))
    }

    fn create_polygon(exterior: Vec<(f64, f64)>) -> Geometry {
        let points: Vec<Point> = exterior.iter().map(|(x, y)| Point::new(*x, *y)).collect();
        let mut ring = points.clone();
        ring.push(points[0]);
        let ls = LineString::new(ring).unwrap();
        Geometry::Polygon(Polygon {
            exterior: ls,
            holes: vec![],
        })
    }

    #[test]
    fn test_mbr_creation() {
        let mbr = MBR::new(0.0, 10.0, 0.0, 10.0);
        assert!(mbr.is_valid());
        assert_eq!(mbr.area(), 100.0);
        assert_eq!(mbr.perimeter(), 40.0);
    }

    #[test]
    fn test_mbr_from_point() {
        let point = Point::new(5.0, 5.0);
        let mbr = MBR::from_point(&point);
        assert_eq!(mbr.min_x, 5.0);
        assert_eq!(mbr.max_x, 5.0);
        assert_eq!(mbr.min_y, 5.0);
        assert_eq!(mbr.max_y, 5.0);
        assert_eq!(mbr.area(), 0.0);
    }

    #[test]
    fn test_mbr_intersects() {
        let mbr1 = MBR::new(0.0, 10.0, 0.0, 10.0);
        let mbr2 = MBR::new(5.0, 15.0, 5.0, 15.0);
        let mbr3 = MBR::new(15.0, 20.0, 15.0, 20.0);

        assert!(mbr1.intersects(&mbr2));
        assert!(!mbr1.intersects(&mbr3));
    }

    #[test]
    fn test_mbr_contains() {
        let outer = MBR::new(0.0, 100.0, 0.0, 100.0);
        let inner = MBR::new(10.0, 50.0, 10.0, 50.0);
        // Truly disjoint - outside the outer bounds
        let disjoint = MBR::new(110.0, 120.0, 110.0, 120.0);

        assert!(outer.contains(&inner));
        assert!(!outer.contains(&disjoint));
    }

    #[test]
    fn test_mbr_contains_point() {
        let mbr = MBR::new(0.0, 10.0, 0.0, 10.0);
        let inside = Point::new(5.0, 5.0);
        let outside = Point::new(15.0, 15.0);
        let on_edge = Point::new(0.0, 5.0);

        assert!(mbr.contains_point(&inside));
        assert!(!mbr.contains_point(&outside));
        assert!(mbr.contains_point(&on_edge));
    }

    #[test]
    fn test_mbr_expand() {
        let mut mbr = MBR::new(0.0, 10.0, 0.0, 10.0);
        let other = MBR::new(5.0, 15.0, 5.0, 15.0);
        mbr.expand(&other);

        assert_eq!(mbr.min_x, 0.0);
        assert_eq!(mbr.max_x, 15.0);
        assert_eq!(mbr.min_y, 0.0);
        assert_eq!(mbr.max_y, 15.0);
    }

    #[test]
    fn test_rtree_empty() {
        let tree = RTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_rtree_insert_point() {
        let mut tree = RTree::new();
        let point = create_point(5.0, 5.0);
        let id = tree.insert(&point);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 1);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_rtree_search_mbr() {
        let mut tree = RTree::new();

        tree.insert(&create_point(5.0, 5.0));
        tree.insert(&create_point(15.0, 15.0));
        tree.insert(&create_point(25.0, 25.0));

        let search_mbr = MBR::new(0.0, 10.0, 0.0, 10.0);
        let results = tree.search(&search_mbr);
        assert_eq!(results.len(), 1);

        let search_mbr = MBR::new(0.0, 20.0, 0.0, 20.0);
        let results = tree.search(&search_mbr);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_rtree_search_contains() {
        let mut tree = RTree::new();

        let poly1 = create_polygon(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        let poly2 = create_polygon(vec![(20.0, 20.0), (30.0, 20.0), (30.0, 30.0), (20.0, 30.0)]);

        tree.insert(&poly1);
        tree.insert(&poly2);

        let point_inside = Point::new(5.0, 5.0);
        let results = tree.search_contains(&point_inside);
        assert_eq!(results.len(), 1);

        let point_inside = Point::new(25.0, 25.0);
        let results = tree.search_contains(&point_inside);
        assert_eq!(results.len(), 1);

        let point_outside = Point::new(15.0, 15.0);
        let results = tree.search_contains(&point_outside);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_rtree_search_intersects() {
        let mut tree = RTree::new();

        let poly1 = create_polygon(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        let poly2 = create_polygon(vec![(5.0, 5.0), (15.0, 5.0), (15.0, 15.0), (5.0, 15.0)]);

        tree.insert(&poly1);
        tree.insert(&poly2);

        let search_poly = create_polygon(vec![(3.0, 3.0), (12.0, 3.0), (12.0, 12.0), (3.0, 12.0)]);
        let results = tree.search_intersects(&search_poly);
        assert_eq!(results.len(), 2);

        let search_poly =
            create_polygon(vec![(50.0, 50.0), (60.0, 50.0), (60.0, 60.0), (50.0, 60.0)]);
        let results = tree.search_intersects(&search_poly);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_rtree_remove() {
        let mut tree = RTree::new();

        let id1 = tree.insert(&create_point(5.0, 5.0));
        tree.insert(&create_point(15.0, 15.0));

        assert!(tree.remove(id1));
        assert_eq!(tree.len(), 1);

        assert!(!tree.remove(id1));
        assert_eq!(tree.len(), 1);

        assert!(tree.remove(id1 + 1));
        assert!(tree.is_empty());
    }

    #[test]
    fn test_rtree_remove_nonexistent() {
        let mut tree = RTree::new();
        tree.insert(&create_point(5.0, 5.0));

        assert!(!tree.remove(999));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_rtree_with_custom_params() {
        let tree = RTree::with_params(8, 3);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_rtree_multiple_insertions() {
        let mut tree = RTree::new();

        for i in 0..10 {
            for j in 0..10 {
                let point = create_point(i as f64 * 10.0, j as f64 * 10.0);
                tree.insert(&point);
            }
        }

        assert_eq!(tree.len(), 100);

        // Search for x in [5, 25], y in [5, 25] to get points at (10,10), (10,20), (20,10), (20,20)
        let search_mbr = MBR::new(5.0, 25.0, 5.0, 25.0);
        let results = tree.search(&search_mbr);
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_rtree_search_after_removal() {
        let mut tree = RTree::new();

        let id1 = tree.insert(&create_point(5.0, 5.0));
        tree.insert(&create_point(15.0, 15.0));
        tree.insert(&create_point(25.0, 25.0));

        tree.remove(id1 + 1);

        let search_mbr = MBR::new(0.0, 30.0, 0.0, 30.0);
        let results = tree.search(&search_mbr);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&id1));
        assert!(results.contains(&(id1 + 2)));
    }

    #[test]
    fn test_rtree_geometry_ids() {
        let mut tree = RTree::new();

        tree.insert(&create_point(5.0, 5.0));
        tree.insert(&create_point(15.0, 15.0));
        tree.insert(&create_point(25.0, 25.0));

        let ids = tree.geometry_ids();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_mbr_union_area() {
        let mbr1 = MBR::new(0.0, 10.0, 0.0, 10.0);
        let mbr2 = MBR::new(5.0, 15.0, 5.0, 15.0);
        let union_area = mbr1.union_area(&mbr2);
        assert_eq!(union_area, 225.0);
    }

    #[test]
    fn test_mbr_margin() {
        let mbr = MBR::new(0.0, 10.0, 0.0, 10.0);
        assert_eq!(mbr.margin(), 40.0);
    }

    #[test]
    fn test_mbr_overlap() {
        let mbr1 = MBR::new(0.0, 10.0, 0.0, 10.0);
        let mbr2 = MBR::new(5.0, 15.0, 5.0, 15.0);
        let mbr3 = MBR::new(15.0, 20.0, 15.0, 20.0);

        assert_eq!(mbr1.overlap(&mbr2), 25.0);
        assert_eq!(mbr1.overlap(&mbr3), 0.0);
    }

    #[test]
    fn test_mbr_distance_to_point() {
        let mbr = MBR::new(10.0, 20.0, 10.0, 20.0);

        let inside = Point::new(15.0, 15.0);
        assert_eq!(mbr.distance_to_point(&inside), 0.0);

        let left = Point::new(5.0, 15.0);
        assert_eq!(mbr.distance_to_point(&left), 5.0);

        let diagonal = Point::new(0.0, 0.0);
        let dist = mbr.distance_to_point(&diagonal);
        assert!((dist - 14.14213562373095).abs() < 0.0001);
    }

    #[test]
    fn test_mbr_display() {
        let mbr = MBR::new(1.0, 2.0, 3.0, 4.0);
        let display = format!("{}", mbr);
        assert!(display.contains("1"));
        assert!(display.contains("2"));
        assert!(display.contains("3"));
        assert!(display.contains("4"));
    }

    #[test]
    fn test_entry_creation() {
        let mbr = MBR::new(0.0, 10.0, 0.0, 10.0);
        let entry = Entry::new(mbr, 42);
        assert_eq!(entry.mbr, mbr);
        assert_eq!(entry.geometry_id, 42);
    }

    #[test]
    fn test_rtree_node_mbr() {
        let leaf = RTreeNode::Leaf {
            entries: vec![
                Entry::new(MBR::new(0.0, 5.0, 0.0, 5.0), 1),
                Entry::new(MBR::new(10.0, 15.0, 10.0, 15.0), 2),
            ],
        };

        let mbr = leaf.mbr();
        assert_eq!(mbr.min_x, 0.0);
        assert_eq!(mbr.max_x, 15.0);
        assert_eq!(mbr.min_y, 0.0);
        assert_eq!(mbr.max_y, 15.0);
    }

    #[test]
    fn test_rtree_node_len() {
        let leaf = RTreeNode::Leaf {
            entries: vec![
                Entry::new(MBR::new(0.0, 5.0, 0.0, 5.0), 1),
                Entry::new(MBR::new(10.0, 15.0, 10.0, 15.0), 2),
            ],
        };

        assert_eq!(leaf.len(), 2);
        assert!(!leaf.is_empty());
    }

    #[test]
    fn test_mbr_infinite() {
        let mbr = MBR::infinite();
        // Infinite MBR is "valid" per is_valid() definition (neg_infinity <= infinity)
        assert!(mbr.is_valid());
        assert!(mbr.area() == f64::INFINITY);
    }
}
