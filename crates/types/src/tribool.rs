//! SQL TriBool - Three-valued logic for SQL NULL handling
//!
//! SQL uses three-valued logic: TRUE, FALSE, and UNKNOWN (NULL)
//!
//! ## Truth Tables
//!
//! ### NOT
//! | p     | NOT p |
//! |-------|-------|
//! | TRUE  | FALSE |
//! | FALSE | TRUE  |
//! | UNKNOWN | UNKNOWN |
//!
//! ### AND
//! | AND   | TRUE | FALSE | UNKNOWN |
//! |-------|------|-------|---------|
//! | TRUE  | TRUE | FALSE | UNKNOWN |
//! | FALSE | FALSE | FALSE | FALSE |
//! | UNKNOWN | UNKNOWN | FALSE | UNKNOWN |
//!
//! ### OR
//! | OR    | TRUE | FALSE | UNKNOWN |
//! |-------|------|-------|---------|
//! | TRUE  | TRUE | TRUE | TRUE |
//! | FALSE | TRUE | FALSE | UNKNOWN |
//! | UNKNOWN | TRUE | UNKNOWN | UNKNOWN |

use serde::{Deserialize, Serialize};

/// SQL TriBool - represents TRUE, FALSE, or UNKNOWN (NULL) in SQL comparisons
///
/// This type implements SQL's three-valued logic for proper NULL handling
/// in predicates, WHERE clauses, and HAVING clauses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriBool {
    /// SQL TRUE
    True,
    /// SQL FALSE
    False,
    /// SQL UNKNOWN (NULL comparison result)
    Unknown,
}

impl TriBool {
    /// SQL unary negation (same as ! operator)
    pub fn negate(self) -> TriBool {
        match self {
            TriBool::True => TriBool::False,
            TriBool::False => TriBool::True,
            TriBool::Unknown => TriBool::Unknown,
        }
    }

    /// Create TriBool from an Option<bool>
    /// None represents UNKNOWN
    pub fn from_option(opt: Option<bool>) -> Self {
        match opt {
            Some(true) => TriBool::True,
            Some(false) => TriBool::False,
            None => TriBool::Unknown,
        }
    }

    /// Convert to Option<bool>, treating UNKNOWN as None
    pub fn to_option(self) -> Option<bool> {
        match self {
            TriBool::True => Some(true),
            TriBool::False => Some(false),
            TriBool::Unknown => None,
        }
    }

    /// SQL AND operation
    pub fn and(self, other: TriBool) -> TriBool {
        match (self, other) {
            (TriBool::False, _) | (_, TriBool::False) => TriBool::False,
            (TriBool::True, TriBool::True) => TriBool::True,
            _ => TriBool::Unknown,
        }
    }

    /// SQL OR operation
    pub fn or(self, other: TriBool) -> TriBool {
        match (self, other) {
            (TriBool::True, _) | (_, TriBool::True) => TriBool::True,
            (TriBool::False, TriBool::False) => TriBool::False,
            _ => TriBool::Unknown,
        }
    }

    /// SQL equality comparison - NULL = NULL returns UNKNOWN (not TRUE!)
    pub fn eq(self, other: TriBool) -> TriBool {
        match (self, other) {
            (TriBool::True, TriBool::True) => TriBool::True,
            (TriBool::False, TriBool::False) => TriBool::True,
            (TriBool::Unknown, TriBool::Unknown) => TriBool::Unknown,
            _ => TriBool::Unknown,
        }
    }

    /// SQL inequality comparison - NULL <> NULL returns UNKNOWN (not FALSE!)
    pub fn ne(self, other: TriBool) -> TriBool {
        self.eq(other).negate()
    }

    /// Greater than comparison
    pub fn gt(self, other: TriBool) -> TriBool {
        match (self, other) {
            (TriBool::True, TriBool::False) => TriBool::True,
            (TriBool::False, TriBool::True) => TriBool::False,
            _ => TriBool::Unknown,
        }
    }

    /// Less than comparison
    pub fn lt(self, other: TriBool) -> TriBool {
        other.gt(self)
    }

    /// Greater than or equal
    pub fn gte(self, other: TriBool) -> TriBool {
        self.eq(other).or(self.gt(other))
    }

    /// Less than or equal
    pub fn lte(self, other: TriBool) -> TriBool {
        self.eq(other).or(self.lt(other))
    }

    /// Convert to SQL predicate result
    /// For WHERE/HAVING: UNKNOWN behaves like FALSE (row is excluded)
    pub fn to_predicate(self) -> bool {
        match self {
            TriBool::True => true,
            TriBool::False | TriBool::Unknown => false,
        }
    }

    /// Create from a boolean value (no UNKNOWN possible)
    pub fn from_bool(b: bool) -> Self {
        if b {
            TriBool::True
        } else {
            TriBool::False
        }
    }

    /// Check if this represents a definitive truth (TRUE or FALSE, not UNKNOWN)
    pub fn is_definitive(self) -> bool {
        match self {
            TriBool::True | TriBool::False => true,
            TriBool::Unknown => false,
        }
    }

    /// Get SQL string representation
    pub fn to_sql_string(self) -> &'static str {
        match self {
            TriBool::True => "TRUE",
            TriBool::False => "FALSE",
            TriBool::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for TriBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_sql_string())
    }
}

impl std::ops::Not for TriBool {
    type Output = TriBool;

    fn not(self) -> TriBool {
        self.negate()
    }
}

impl std::ops::BitAnd for TriBool {
    type Output = TriBool;

    fn bitand(self, other: TriBool) -> TriBool {
        self.and(other)
    }
}

impl std::ops::BitOr for TriBool {
    type Output = TriBool;

    fn bitor(self, other: TriBool) -> TriBool {
        self.or(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOT truth table tests
    #[test]
    fn test_not_true() {
        assert_eq!(!TriBool::True, TriBool::False);
    }

    #[test]
    fn test_not_false() {
        assert_eq!(!TriBool::False, TriBool::True);
    }

    #[test]
    fn test_not_unknown() {
        assert_eq!(!TriBool::Unknown, TriBool::Unknown);
    }

    // AND truth table tests
    #[test]
    fn test_and_false_false() {
        assert_eq!(TriBool::False.and(TriBool::False), TriBool::False);
    }

    #[test]
    fn test_and_false_true() {
        assert_eq!(TriBool::False.and(TriBool::True), TriBool::False);
    }

    #[test]
    fn test_and_true_false() {
        assert_eq!(TriBool::True.and(TriBool::False), TriBool::False);
    }

    #[test]
    fn test_and_true_true() {
        assert_eq!(TriBool::True.and(TriBool::True), TriBool::True);
    }

    #[test]
    fn test_and_with_unknown() {
        assert_eq!(TriBool::True.and(TriBool::Unknown), TriBool::Unknown);
        assert_eq!(TriBool::Unknown.and(TriBool::True), TriBool::Unknown);
        assert_eq!(TriBool::False.and(TriBool::Unknown), TriBool::False);
        assert_eq!(TriBool::Unknown.and(TriBool::False), TriBool::False);
        assert_eq!(TriBool::Unknown.and(TriBool::Unknown), TriBool::Unknown);
    }

    // OR truth table tests
    #[test]
    fn test_or_true_true() {
        assert_eq!(TriBool::True.or(TriBool::True), TriBool::True);
    }

    #[test]
    fn test_or_true_false() {
        assert_eq!(TriBool::True.or(TriBool::False), TriBool::True);
    }

    #[test]
    fn test_or_false_true() {
        assert_eq!(TriBool::False.or(TriBool::True), TriBool::True);
    }

    #[test]
    fn test_or_false_false() {
        assert_eq!(TriBool::False.or(TriBool::False), TriBool::False);
    }

    #[test]
    fn test_or_with_unknown() {
        assert_eq!(TriBool::False.or(TriBool::Unknown), TriBool::Unknown);
        assert_eq!(TriBool::Unknown.or(TriBool::False), TriBool::Unknown);
        assert_eq!(TriBool::True.or(TriBool::Unknown), TriBool::True);
        assert_eq!(TriBool::Unknown.or(TriBool::True), TriBool::True);
        assert_eq!(TriBool::Unknown.or(TriBool::Unknown), TriBool::Unknown);
    }

    // NULL = NULL returns UNKNOWN
    #[test]
    fn test_null_equals_null_is_unknown() {
        assert_eq!(TriBool::Unknown.eq(TriBool::Unknown), TriBool::Unknown);
    }

    #[test]
    fn test_null_not_equals_null_is_unknown() {
        assert_eq!(TriBool::Unknown.ne(TriBool::Unknown), TriBool::Unknown);
    }

    // to_predicate: UNKNOWN becomes false for WHERE filtering
    #[test]
    fn test_to_predicate_true() {
        assert_eq!(TriBool::True.to_predicate(), true);
    }

    #[test]
    fn test_to_predicate_false() {
        assert_eq!(TriBool::False.to_predicate(), false);
    }

    #[test]
    fn test_to_predicate_unknown() {
        assert_eq!(TriBool::Unknown.to_predicate(), false); // UNKNOWN → FALSE in WHERE
    }

    // Option conversion
    #[test]
    fn test_from_option_some_true() {
        assert_eq!(TriBool::from_option(Some(true)), TriBool::True);
    }

    #[test]
    fn test_from_option_some_false() {
        assert_eq!(TriBool::from_option(Some(false)), TriBool::False);
    }

    #[test]
    fn test_from_option_none() {
        assert_eq!(TriBool::from_option(None), TriBool::Unknown);
    }

    #[test]
    fn test_to_option_true() {
        assert_eq!(TriBool::True.to_option(), Some(true));
    }

    #[test]
    fn test_to_option_false() {
        assert_eq!(TriBool::False.to_option(), Some(false));
    }

    #[test]
    fn test_to_option_unknown() {
        assert_eq!(TriBool::Unknown.to_option(), None);
    }

    // Operator overloads
    #[test]
    fn test_not_operator() {
        assert_eq!(!TriBool::True, TriBool::False);
        assert_eq!(!TriBool::False, TriBool::True);
        assert_eq!(!TriBool::Unknown, TriBool::Unknown);
    }

    #[test]
    fn test_and_operator() {
        assert_eq!(TriBool::True & TriBool::True, TriBool::True);
        assert_eq!(TriBool::True & TriBool::False, TriBool::False);
        assert_eq!(TriBool::True & TriBool::Unknown, TriBool::Unknown);
    }

    #[test]
    fn test_or_operator() {
        assert_eq!(TriBool::False | TriBool::False, TriBool::False);
        assert_eq!(TriBool::False | TriBool::True, TriBool::True);
        assert_eq!(TriBool::False | TriBool::Unknown, TriBool::Unknown);
    }

    // SQL string representation
    #[test]
    fn test_to_sql_string() {
        assert_eq!(TriBool::True.to_sql_string(), "TRUE");
        assert_eq!(TriBool::False.to_sql_string(), "FALSE");
        assert_eq!(TriBool::Unknown.to_sql_string(), "UNKNOWN");
    }

    // Display
    #[test]
    fn test_display() {
        assert_eq!(format!("{}", TriBool::True), "TRUE");
        assert_eq!(format!("{}", TriBool::False), "FALSE");
        assert_eq!(format!("{}", TriBool::Unknown), "UNKNOWN");
    }
}
