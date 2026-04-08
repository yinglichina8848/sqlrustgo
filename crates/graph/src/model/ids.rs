//! Graph ID types
//!
//! Internal ID system using u64/u32 for performance.
//! UUID is only used as a property field, not for internal referencing.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign};

/// Node identifier - internal use u64 for performance
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct NodeId(pub u64);

impl NodeId {
    pub const MIN: NodeId = NodeId(0);
    pub const MAX: NodeId = NodeId(u64::MAX);

    #[inline]
    pub fn new(id: u64) -> Self {
        NodeId(id)
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn next(self) -> NodeId {
        NodeId(self.0 + 1)
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<u64> for NodeId {
    type Output = NodeId;
    fn add(self, rhs: u64) -> NodeId {
        NodeId(self.0 + rhs)
    }
}

impl AddAssign<u64> for NodeId {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

/// Edge identifier - internal use u64 for performance
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct EdgeId(pub u64);

impl EdgeId {
    pub const MIN: EdgeId = EdgeId(0);
    pub const MAX: EdgeId = EdgeId(u64::MAX);

    #[inline]
    pub fn new(id: u64) -> Self {
        EdgeId(id)
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn next(self) -> EdgeId {
        EdgeId(self.0 + 1)
    }
}

impl fmt::Debug for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EdgeId({})", self.0)
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<u64> for EdgeId {
    type Output = EdgeId;
    fn add(self, rhs: u64) -> EdgeId {
        EdgeId(self.0 + rhs)
    }
}

impl AddAssign<u64> for EdgeId {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

/// Label identifier - u32 is sufficient for labels
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct LabelId(pub u32);

impl LabelId {
    pub const MIN: LabelId = LabelId(0);
    pub const MAX: LabelId = LabelId(u32::MAX);

    #[inline]
    pub fn new(id: u32) -> Self {
        LabelId(id)
    }

    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn next(self) -> LabelId {
        LabelId(self.0 + 1)
    }
}

impl fmt::Debug for LabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LabelId({})", self.0)
    }
}

impl fmt::Display for LabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<u32> for LabelId {
    type Output = LabelId;
    fn add(self, rhs: u32) -> LabelId {
        LabelId(self.0 + rhs)
    }
}

impl AddAssign<u32> for LabelId {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(42);
        assert_eq!(id.as_u64(), 42);
        assert_eq!(id, NodeId(42));
        assert_eq!(format!("{:?}", id), "NodeId(42)");
        assert_eq!(format!("{}", id), "42");
    }

    #[test]
    fn test_node_id_operations() {
        let id = NodeId::new(10);
        assert_eq!(id.next(), NodeId(11));
        assert_eq!(id + 5, NodeId(15));
        let mut id_mut = NodeId::new(10);
        id_mut += 5;
        assert_eq!(id_mut, NodeId(15));
    }

    #[test]
    fn test_edge_id() {
        let id = EdgeId::new(99);
        assert_eq!(id.as_u64(), 99);
        assert_eq!(id, EdgeId(99));
        assert_eq!(format!("{:?}", id), "EdgeId(99)");
    }

    #[test]
    fn test_edge_id_operations() {
        let id = EdgeId::new(10);
        assert_eq!(id.next(), EdgeId(11));
        assert_eq!(id + 5, EdgeId(15));
    }

    #[test]
    fn test_label_id() {
        let id = LabelId::new(5);
        assert_eq!(id.as_u32(), 5);
        assert_eq!(id, LabelId(5));
        assert_eq!(format!("{:?}", id), "LabelId(5)");
    }

    #[test]
    fn test_label_id_operations() {
        let id = LabelId::new(10);
        assert_eq!(id.next(), LabelId(11));
        assert_eq!(id + 5, LabelId(15));
    }

    #[test]
    fn test_id_equality() {
        assert_eq!(NodeId(1), NodeId(1));
        assert_ne!(NodeId(1), NodeId(2));
        assert_eq!(EdgeId(1), EdgeId(1));
        assert_eq!(LabelId(1), LabelId(1));
    }

    #[test]
    fn test_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NodeId(1));
        set.insert(NodeId(2));
        set.insert(NodeId(1)); // duplicate
        assert_eq!(set.len(), 2);
    }
}
