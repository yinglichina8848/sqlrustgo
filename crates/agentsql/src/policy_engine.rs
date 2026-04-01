use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub resource: String,
    pub actions: Vec<String>,
    pub conditions: Vec<PolicyCondition>,
    pub effect: PolicyEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionOperator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    In,
    NotIn,
    Like,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckRequest {
    pub user_id: String,
    pub resource: String,
    pub action: String,
    pub context: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckResponse {
    pub allowed: bool,
    pub matched_policy: Option<String>,
    pub reason: String,
}

pub struct PolicyEngine {
    policies: Vec<Policy>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        let policies = Self::default_policies();
        Self { policies }
    }

    fn default_policies() -> Vec<Policy> {
        vec![
            Policy {
                id: "policy_read_all".to_string(),
                name: "Read All Tables".to_string(),
                resource: "table:*".to_string(),
                actions: vec!["SELECT".to_string()],
                conditions: vec![],
                effect: PolicyEffect::Allow,
            },
            Policy {
                id: "policy_write_restricted".to_string(),
                name: "Restricted Write".to_string(),
                resource: "table:sensitive_data".to_string(),
                actions: vec![
                    "INSERT".to_string(),
                    "UPDATE".to_string(),
                    "DELETE".to_string(),
                ],
                conditions: vec![PolicyCondition {
                    field: "user_role".to_string(),
                    operator: ConditionOperator::Eq,
                    value: serde_json::json!("admin"),
                }],
                effect: PolicyEffect::Allow,
            },
            Policy {
                id: "policy_deny_delete".to_string(),
                name: "Deny Delete on Audit".to_string(),
                resource: "table:audit_log".to_string(),
                actions: vec!["DELETE".to_string()],
                conditions: vec![],
                effect: PolicyEffect::Deny,
            },
        ]
    }

    pub fn check(&self, request: &PolicyCheckRequest) -> PolicyCheckResponse {
        for policy in &self.policies {
            if self.policy_matches(policy, request) {
                return PolicyCheckResponse {
                    allowed: policy.effect == PolicyEffect::Allow,
                    matched_policy: Some(policy.id.clone()),
                    reason: format!("Matched policy: {}", policy.name),
                };
            }
        }

        PolicyCheckResponse {
            allowed: false,
            matched_policy: None,
            reason: "No matching policy found".to_string(),
        }
    }

    fn policy_matches(&self, policy: &Policy, request: &PolicyCheckRequest) -> bool {
        if !Self::resource_matches(&policy.resource, &request.resource) {
            return false;
        }

        if !policy.actions.contains(&request.action) && !policy.actions.contains(&"*".to_string()) {
            return false;
        }

        if let Some(ref ctx) = request.context {
            for condition in &policy.conditions {
                if !Self::condition_matches(condition, ctx) {
                    return false;
                }
            }
        } else if !policy.conditions.is_empty() {
            return false;
        }

        true
    }

    fn resource_matches(policy_resource: &str, request_resource: &str) -> bool {
        if policy_resource == "*" || policy_resource == request_resource {
            return true;
        }

        if policy_resource.ends_with(":*") {
            let prefix = &policy_resource[..policy_resource.len() - 2];
            return request_resource.starts_with(prefix);
        }

        false
    }

    fn condition_matches(
        condition: &PolicyCondition,
        context: &HashMap<String, serde_json::Value>,
    ) -> bool {
        let Some(value) = context.get(&condition.field) else {
            return false;
        };

        match condition.operator {
            ConditionOperator::Eq => value == &condition.value,
            ConditionOperator::Ne => value != &condition.value,
            ConditionOperator::Gt => Self::compare_numeric(value, &condition.value) > 0,
            ConditionOperator::Lt => Self::compare_numeric(value, &condition.value) < 0,
            ConditionOperator::Gte => Self::compare_numeric(value, &condition.value) >= 0,
            ConditionOperator::Lte => Self::compare_numeric(value, &condition.value) <= 0,
            ConditionOperator::In => {
                if let Some(arr) = condition.value.as_array() {
                    arr.contains(value)
                } else {
                    false
                }
            }
            ConditionOperator::NotIn => {
                if let Some(arr) = condition.value.as_array() {
                    !arr.contains(value)
                } else {
                    true
                }
            }
            ConditionOperator::Like => {
                if let (Some(v), Some(p)) = (value.as_str(), condition.value.as_str()) {
                    Self::simple_like(v, p)
                } else {
                    false
                }
            }
            ConditionOperator::IsNull => value.is_null(),
            ConditionOperator::IsNotNull => !value.is_null(),
        }
    }

    fn compare_numeric(a: &serde_json::Value, b: &serde_json::Value) -> i32 {
        let a_f = a.as_f64().unwrap_or(0.0);
        let b_f = b.as_f64().unwrap_or(0.0);
        a_f.partial_cmp(&b_f).map(|o| o as i32).unwrap_or(0)
    }

    fn simple_like(text: &str, pattern: &str) -> bool {
        let pattern = pattern.replace('%', ".*").replace('_', ".");
        regex_lite::Regex::new(&format!("^{}$", pattern))
            .map(|r| r.is_match(text))
            .unwrap_or(false)
    }

    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    pub fn list_policies(&self) -> Vec<&Policy> {
        self.policies.iter().collect()
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine_check() {
        let engine = PolicyEngine::new();

        let request = PolicyCheckRequest {
            user_id: "user1".to_string(),
            resource: "table:users".to_string(),
            action: "SELECT".to_string(),
            context: None,
        };

        let response = engine.check(&request);
        assert!(response.allowed);
        assert_eq!(response.matched_policy, Some("policy_read_all".to_string()));
    }

    #[test]
    fn test_policy_engine_deny() {
        let engine = PolicyEngine::new();

        let request = PolicyCheckRequest {
            user_id: "user1".to_string(),
            resource: "table:audit_log".to_string(),
            action: "DELETE".to_string(),
            context: None,
        };

        let response = engine.check(&request);
        assert!(!response.allowed);
        assert_eq!(
            response.matched_policy,
            Some("policy_deny_delete".to_string())
        );
    }

    #[test]
    fn test_policy_engine_with_conditions() {
        let engine = PolicyEngine::new();

        let mut context = HashMap::new();
        context.insert("user_role".to_string(), serde_json::json!("admin"));

        let request = PolicyCheckRequest {
            user_id: "admin1".to_string(),
            resource: "table:sensitive_data".to_string(),
            action: "INSERT".to_string(),
            context: Some(context),
        };

        let response = engine.check(&request);
        assert!(response.allowed);
    }
}
