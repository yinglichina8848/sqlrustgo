use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingRule {
    pub id: String,
    pub column: String,
    pub mask_type: MaskingType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskingType {
    Full,
    Partial,
    Hash,
    Truncate,
    Null,
    Range,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingConfig {
    pub rules: Vec<MaskingRule>,
}

impl Default for MaskingConfig {
    fn default() -> Self {
        Self {
            rules: vec![
                MaskingRule {
                    id: "mask_email".to_string(),
                    column: "email".to_string(),
                    mask_type: MaskingType::Partial,
                    description: "Mask email addresses".to_string(),
                },
                MaskingRule {
                    id: "mask_phone".to_string(),
                    column: "phone".to_string(),
                    mask_type: MaskingType::Partial,
                    description: "Mask phone numbers".to_string(),
                },
                MaskingRule {
                    id: "mask_ssn".to_string(),
                    column: "ssn".to_string(),
                    mask_type: MaskingType::Hash,
                    description: "Hash SSN values".to_string(),
                },
                MaskingRule {
                    id: "mask_credit_card".to_string(),
                    column: "credit_card".to_string(),
                    mask_type: MaskingType::Full,
                    description: "Fully mask credit card numbers".to_string(),
                },
                MaskingRule {
                    id: "mask_salary".to_string(),
                    column: "salary".to_string(),
                    mask_type: MaskingType::Range,
                    description: "Mask salary to ranges".to_string(),
                },
            ],
        }
    }
}

pub struct ColumnMasker {
    config: MaskingConfig,
}

impl ColumnMasker {
    pub fn new() -> Self {
        Self {
            config: MaskingConfig::default(),
        }
    }

    pub fn new_with_config(config: MaskingConfig) -> Self {
        Self { config }
    }

    pub fn mask_value(&self, column: &str, value: &serde_json::Value) -> serde_json::Value {
        if let Some(rule) = self.find_rule(column) {
            self.apply_mask(rule, value)
        } else {
            value.clone()
        }
    }

    pub fn mask_row(
        &self,
        row: &HashMap<String, serde_json::Value>,
        columns: &[String],
    ) -> HashMap<String, serde_json::Value> {
        let mut masked_row = row.clone();
        for col in columns {
            if let Some(value) = row.get(col) {
                let masked_value = self.mask_value(col, value);
                masked_row.insert(col.clone(), masked_value);
            }
        }
        masked_row
    }

    pub fn mask_result_set(
        &self,
        rows: &[Vec<serde_json::Value>],
        columns: &[String],
    ) -> Vec<Vec<serde_json::Value>> {
        rows.iter()
            .map(|row| {
                columns
                    .iter()
                    .enumerate()
                    .map(|(i, col)| {
                        row.get(i)
                            .map(|v| self.mask_value(col, v))
                            .unwrap_or(serde_json::Value::Null)
                    })
                    .collect()
            })
            .collect()
    }

    fn find_rule(&self, column: &str) -> Option<&MaskingRule> {
        self.config
            .rules
            .iter()
            .find(|r| r.column == column || column.ends_with(&format!(".{}", r.column)))
    }

    fn apply_mask(&self, rule: &MaskingRule, value: &serde_json::Value) -> serde_json::Value {
        match rule.mask_type {
            MaskingType::Full => self.mask_full(value),
            MaskingType::Partial => self.mask_partial(value),
            MaskingType::Hash => self.mask_hash(value),
            MaskingType::Truncate => self.mask_truncate(value),
            MaskingType::Null => serde_json::Value::Null,
            MaskingType::Range => self.mask_range(value),
            MaskingType::Custom => value.clone(),
        }
    }

    fn mask_full(&self, value: &serde_json::Value) -> serde_json::Value {
        if let Some(_s) = value.as_str() {
            serde_json::json!("****")
        } else if value.is_number() {
            serde_json::json!(0)
        } else {
            serde_json::Value::Null
        }
    }

    fn mask_partial(&self, value: &serde_json::Value) -> serde_json::Value {
        if let Some(s) = value.as_str() {
            let len = s.len();
            if len <= 2 {
                serde_json::json!("**")
            } else if len <= 4 {
                serde_json::json!(format!("*{}", &s[len - 1..]))
            } else {
                let visible = (len / 4).max(1);
                serde_json::json!(format!(
                    "{}{}{}",
                    &s[..visible],
                    "*".repeat(len - visible * 2),
                    &s[len - visible..]
                ))
            }
        } else {
            value.clone()
        }
    }

    fn mask_hash(&self, value: &serde_json::Value) -> serde_json::Value {
        use std::hash::{DefaultHasher, Hash, Hasher};

        if let Some(s) = value.as_str() {
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            serde_json::json!(format!("{:x}", hasher.finish()))
        } else if let Some(n) = value.as_i64() {
            let mut hasher = DefaultHasher::new();
            n.hash(&mut hasher);
            serde_json::json!(format!("{:x}", hasher.finish()))
        } else {
            value.clone()
        }
    }

    fn mask_truncate(&self, value: &serde_json::Value) -> serde_json::Value {
        if let Some(s) = value.as_str() {
            if s.len() <= 3 {
                serde_json::json!(s)
            } else {
                serde_json::json!(&s[..3])
            }
        } else {
            value.clone()
        }
    }

    fn mask_range(&self, value: &serde_json::Value) -> serde_json::Value {
        if let Some(n) = value.as_f64() {
            let range = match n {
                n if n < 10000.0 => "< 10K".to_string(),
                n if n < 50000.0 => "10K - 50K".to_string(),
                n if n < 100000.0 => "50K - 100K".to_string(),
                _ => "> 100K".to_string(),
            };
            serde_json::json!(range)
        } else {
            value.clone()
        }
    }

    pub fn add_rule(&mut self, rule: MaskingRule) {
        self.config.rules.push(rule);
    }

    pub fn list_rules(&self) -> Vec<&MaskingRule> {
        self.config.rules.iter().collect()
    }
}

impl Default for ColumnMasker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_partial_email() {
        let masker = ColumnMasker::new();
        let value = serde_json::json!("user@example.com");
        let masked = masker.mask_value("email", &value);
        let masked_str = masked.as_str().unwrap();
        assert!(masked_str.contains('*'), "Should contain mask character");
        assert_ne!(masked_str, "user@example.com", "Should be masked");
    }

    #[test]
    fn test_mask_hash() {
        let masker = ColumnMasker::new();
        let value = serde_json::json!("123-45-6789");
        let masked = masker.mask_value("ssn", &value);
        assert!(masked.is_string());
        assert_ne!(masked, value);
    }

    #[test]
    fn test_mask_full() {
        let masker = ColumnMasker::new();
        let value = serde_json::json!("4532123456789012");
        let masked = masker.mask_value("credit_card", &value);
        assert_eq!(masked, serde_json::json!("****"));
    }

    #[test]
    fn test_mask_range() {
        let masker = ColumnMasker::new();
        let masked = masker.mask_value("salary", &serde_json::json!(75000.0));
        assert_eq!(masked, serde_json::json!("50K - 100K"));
    }

    #[test]
    fn test_mask_null() {
        let masker = ColumnMasker::new();
        let value = serde_json::json!("secret");
        let masked = masker.mask_value("non_existent_column", &value);
        assert_eq!(masked, value);
    }
}
