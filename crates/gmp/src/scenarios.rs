//! GMP Top 10 Scenarios Implementation
//!
//! This module implements the GMP (Graph Memory Processor) Top 10 scenarios
//! as defined in the v2.7.0 VERSION_PLAN.md.
//!
//! # Scenarios
//!
//! 1. Social Network Friend Recommendation (P0)
//! 2. Knowledge Graph Q&A (P0)
//! 3. Fraud Detection (P0)
//! 4. Recommendation System (P1)
//! 5. Supply Chain Tracking (P1)
//! 6. Organization Analysis (P1)
//! 7. Security Threat Analysis (P2)
//! 8. Bioinformatics Retrieval (P2)
//! 9. Financial Risk Control (P2)
//! 10. Logistics Optimization (P2)

use serde::{Deserialize, Serialize};

// =============================================================================
// Scenario Definitions
// =============================================================================

/// GMP Scenario identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum GmpScenario {
    /// Social Network Friend Recommendation
    SocialFriendRecommendation = 1,
    /// Knowledge Graph Q&A
    KnowledgeGraphQA = 2,
    /// Fraud Detection
    FraudDetection = 3,
    /// Recommendation System
    RecommendationSystem = 4,
    /// Supply Chain Tracking
    SupplyChainTracking = 5,
    /// Organization Analysis
    OrganizationAnalysis = 6,
    /// Security Threat Analysis
    SecurityThreatAnalysis = 7,
    /// Bioinformatics Retrieval
    BioinformaticsRetrieval = 8,
    /// FinancialRiskControl
    FinancialRiskControl = 9,
    /// Logistics Optimization
    LogisticsOptimization = 10,
}

impl GmpScenario {
    /// Get scenario name
    pub fn name(&self) -> &'static str {
        match self {
            GmpScenario::SocialFriendRecommendation => "Social Friend Recommendation",
            GmpScenario::KnowledgeGraphQA => "Knowledge Graph Q&A",
            GmpScenario::FraudDetection => "Fraud Detection",
            GmpScenario::RecommendationSystem => "Recommendation System",
            GmpScenario::SupplyChainTracking => "Supply Chain Tracking",
            GmpScenario::OrganizationAnalysis => "Organization Analysis",
            GmpScenario::SecurityThreatAnalysis => "Security Threat Analysis",
            GmpScenario::BioinformaticsRetrieval => "Bioinformatics Retrieval",
            GmpScenario::FinancialRiskControl => "Financial Risk Control",
            GmpScenario::LogisticsOptimization => "Logistics Optimization",
        }
    }

    /// Get scenario priority
    pub fn priority(&self) -> &'static str {
        match self {
            GmpScenario::SocialFriendRecommendation => "P0",
            GmpScenario::KnowledgeGraphQA => "P0",
            GmpScenario::FraudDetection => "P0",
            GmpScenario::RecommendationSystem => "P1",
            GmpScenario::SupplyChainTracking => "P1",
            GmpScenario::OrganizationAnalysis => "P1",
            GmpScenario::SecurityThreatAnalysis => "P2",
            GmpScenario::BioinformaticsRetrieval => "P2",
            GmpScenario::FinancialRiskControl => "P2",
            GmpScenario::LogisticsOptimization => "P2",
        }
    }

    /// Get required graph patterns for this scenario
    pub fn required_patterns(&self) -> Vec<&'static str> {
        match self {
            GmpScenario::SocialFriendRecommendation => {
                vec!["(a)-[:friendship]->(b)", "(b)-[:friendship]->(c)"]
            }
            GmpScenario::KnowledgeGraphQA => {
                vec!["(e1)-[:relation]->(e2)", "(e2)-[:relation]->(e3)"]
            }
            GmpScenario::FraudDetection => {
                vec![
                    "(a)-[:transaction]->(b)",
                    "(b)-[:transaction]->(c)",
                    "(a)-[:account]->(x)",
                ]
            }
            GmpScenario::RecommendationSystem => {
                vec!["(user)-[:prefers]->(item)", "(item)-[:category]->(cat)"]
            }
            GmpScenario::SupplyChainTracking => {
                vec!["(p)-[:supplied_by]->(s)", "(s)-[:supplied_by]->(s2)"]
            }
            GmpScenario::OrganizationAnalysis => {
                vec!["(e)-[:reports_to]->(m)", "(m)-[:reports_to]->(m2)"]
            }
            GmpScenario::SecurityThreatAnalysis => {
                vec!["(h1)-[:connects]->(h2)", "(h2)-[:exploits]->(v)"]
            }
            GmpScenario::BioinformaticsRetrieval => {
                vec!["(p1)-[:interacts_with]->(p2)", "(p2)-[:pathway]->(pw)"]
            }
            GmpScenario::FinancialRiskControl => {
                vec!["(c1)-[:guarantees]->(c2)", "(c2)-[:guarantees]->(c3)"]
            }
            GmpScenario::LogisticsOptimization => {
                vec!["(l1)-[:route_to]->(l2)", "(l2)-[:route_to]->(l3)"]
            }
        }
    }
}

// =============================================================================
// Graph Pattern Types
// =============================================================================

/// Vertex type in graph pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub id: String,
    pub label: String,
    pub properties: Vec<(String, String)>,
}

/// Edge type in graph pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub properties: Vec<(String, String)>,
}

/// Graph pattern for GMP queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPattern {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

impl GraphPattern {
    /// Create a simple two-hop friendship pattern
    pub fn two_hop_friendship(start: &str, middle: &str, end: &str) -> Self {
        Self {
            vertices: vec![
                Vertex {
                    id: start.to_string(),
                    label: "User".to_string(),
                    properties: vec![],
                },
                Vertex {
                    id: middle.to_string(),
                    label: "User".to_string(),
                    properties: vec![],
                },
                Vertex {
                    id: end.to_string(),
                    label: "User".to_string(),
                    properties: vec![],
                },
            ],
            edges: vec![
                Edge {
                    from: start.to_string(),
                    to: middle.to_string(),
                    label: "friendship".to_string(),
                    properties: vec![],
                },
                Edge {
                    from: middle.to_string(),
                    to: end.to_string(),
                    label: "friendship".to_string(),
                    properties: vec![],
                },
            ],
        }
    }
}

// =============================================================================
// Query Builders
// =============================================================================

/// GMP query builder for scenarios
#[derive(Debug, Clone)]
pub struct GmpQueryBuilder {
    scenario: GmpScenario,
    pattern: GraphPattern,
    filters: Vec<String>,
    limit: Option<usize>,
}

impl GmpQueryBuilder {
    /// Create a new query builder for a scenario
    pub fn new(scenario: GmpScenario) -> Self {
        let pattern = Self::default_pattern_for(scenario);
        Self {
            scenario,
            pattern,
            filters: Vec::new(),
            limit: None,
        }
    }

    /// Get default graph pattern for scenario
    fn default_pattern_for(scenario: GmpScenario) -> GraphPattern {
        match scenario {
            GmpScenario::SocialFriendRecommendation => {
                GraphPattern::two_hop_friendship("me", "friend", "fof")
            }
            GmpScenario::KnowledgeGraphQA => GraphPattern {
                vertices: vec![
                    Vertex {
                        id: "e1".to_string(),
                        label: "Entity".to_string(),
                        properties: vec![],
                    },
                    Vertex {
                        id: "e2".to_string(),
                        label: "Entity".to_string(),
                        properties: vec![],
                    },
                    Vertex {
                        id: "e3".to_string(),
                        label: "Entity".to_string(),
                        properties: vec![],
                    },
                ],
                edges: vec![
                    Edge {
                        from: "e1".to_string(),
                        to: "e2".to_string(),
                        label: "relation".to_string(),
                        properties: vec![],
                    },
                    Edge {
                        from: "e2".to_string(),
                        to: "e3".to_string(),
                        label: "relation".to_string(),
                        properties: vec![],
                    },
                ],
            },
            GmpScenario::FraudDetection => GraphPattern {
                vertices: vec![
                    Vertex {
                        id: "a".to_string(),
                        label: "Account".to_string(),
                        properties: vec![],
                    },
                    Vertex {
                        id: "b".to_string(),
                        label: "Account".to_string(),
                        properties: vec![],
                    },
                    Vertex {
                        id: "c".to_string(),
                        label: "Account".to_string(),
                        properties: vec![],
                    },
                ],
                edges: vec![
                    Edge {
                        from: "a".to_string(),
                        to: "b".to_string(),
                        label: "transaction".to_string(),
                        properties: vec![],
                    },
                    Edge {
                        from: "b".to_string(),
                        to: "c".to_string(),
                        label: "transaction".to_string(),
                        properties: vec![],
                    },
                ],
            },
            _ => GraphPattern {
                vertices: vec![],
                edges: vec![],
            },
        }
    }

    /// Add a filter condition
    pub fn filter(mut self, condition: &str) -> Self {
        self.filters.push(condition.to_string());
        self
    }

    /// Set result limit
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Build GMP MATCH clause
    pub fn build_match_clause(&self) -> String {
        let edge_clauses: Vec<String> = self
            .pattern
            .edges
            .iter()
            .map(|e| format!("({})-[:{}]->({})", e.from, e.label, e.to))
            .collect();

        format!("GRAPH MATCH {}", edge_clauses.join(", "))
    }

    /// Build WHERE clause
    pub fn build_where_clause(&self) -> Option<String> {
        if self.filters.is_empty() {
            None
        } else {
            Some(format!("WHERE {}", self.filters.join(" AND ")))
        }
    }

    /// Build complete GMP query
    pub fn build(&self) -> String {
        let mut query = self.build_match_clause();

        if let Some(where_clause) = self.build_where_clause() {
            query.push_str(&format!(" {}", where_clause));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        query
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_names() {
        assert_eq!(
            GmpScenario::SocialFriendRecommendation.name(),
            "Social Friend Recommendation"
        );
        assert_eq!(GmpScenario::FraudDetection.name(), "Fraud Detection");
    }

    #[test]
    fn test_scenario_priority() {
        assert_eq!(GmpScenario::SocialFriendRecommendation.priority(), "P0");
        assert_eq!(GmpScenario::RecommendationSystem.priority(), "P1");
        assert_eq!(GmpScenario::SecurityThreatAnalysis.priority(), "P2");
    }

    #[test]
    fn test_two_hop_friendship_pattern() {
        let pattern = GraphPattern::two_hop_friendship("me", "f1", "f2");
        assert_eq!(pattern.vertices.len(), 3);
        assert_eq!(pattern.edges.len(), 2);
        assert_eq!(pattern.edges[0].label, "friendship");
        assert_eq!(pattern.edges[1].label, "friendship");
    }

    #[test]
    fn test_query_builder_two_hop() {
        let query = GmpQueryBuilder::new(GmpScenario::SocialFriendRecommendation)
            .filter("me.id = 'user123'")
            .filter("fof.id != 'user123'")
            .limit(10)
            .build();

        assert!(query.contains("GRAPH MATCH"));
        assert!(query.contains("(me)-[:friendship]->(friend)"));
        assert!(query.contains("(friend)-[:friendship]->(fof)"));
        assert!(query.contains("WHERE"));
        assert!(query.contains("LIMIT 10"));
    }

    #[test]
    fn test_query_builder_fraud_detection() {
        let query = GmpQueryBuilder::new(GmpScenario::FraudDetection)
            .filter("a.id = 'suspect_account'")
            .filter("SUM(t.amount) > 10000")
            .limit(100)
            .build();

        assert!(query.contains("GRAPH MATCH"));
        assert!(query.contains("(a)-[:transaction]->(b)"));
        assert!(query.contains("LIMIT 100"));
    }

    #[test]
    fn test_knowledge_graph_multihop() {
        let scenario = GmpScenario::KnowledgeGraphQA;
        let patterns = scenario.required_patterns();

        assert!(patterns.contains(&"(e1)-[:relation]->(e2)"));
        assert!(patterns.contains(&"(e2)-[:relation]->(e3)"));
    }

    #[test]
    fn test_all_scenarios_have_patterns() {
        let scenarios: Vec<GmpScenario> = vec![
            GmpScenario::SocialFriendRecommendation,
            GmpScenario::KnowledgeGraphQA,
            GmpScenario::FraudDetection,
            GmpScenario::RecommendationSystem,
            GmpScenario::SupplyChainTracking,
            GmpScenario::OrganizationAnalysis,
            GmpScenario::SecurityThreatAnalysis,
            GmpScenario::BioinformaticsRetrieval,
            GmpScenario::FinancialRiskControl,
            GmpScenario::LogisticsOptimization,
        ];

        for scenario in scenarios {
            let patterns = scenario.required_patterns();
            assert!(
                !patterns.is_empty(),
                "Scenario {:?} should have patterns",
                scenario
            );
        }
    }
}
