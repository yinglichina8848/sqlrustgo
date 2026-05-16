use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablePolicy {
    pub table_name: String,
    pub predicate: String,
}

#[derive(Debug, Clone, Default)]
pub struct RowLevelSecurity {
    policies: HashMap<u64, Vec<TablePolicy>>,
}

impl RowLevelSecurity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_policy(&mut self, user_id: u64, table: &str, predicate: &str) {
        self.policies
            .entry(user_id)
            .or_default()
            .push(TablePolicy {
                table_name: table.to_string(),
                predicate: predicate.to_string(),
            });
    }

    pub fn get_predicate(&self, user_id: u64, table: &str) -> Option<String> {
        self.policies
            .get(&user_id)
            .and_then(|policies| {
                policies
                    .iter()
                    .find(|p| p.table_name == table)
                    .map(|p| p.predicate.clone())
            })
    }

    pub fn has_policy(&self, user_id: u64, table: &str) -> bool {
        self.get_predicate(user_id, table).is_some()
    }

    pub fn get_user_policies(&self, user_id: u64) -> Vec<TablePolicy> {
        self.policies.get(&user_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_policy() {
        let mut rls = RowLevelSecurity::new();
        rls.add_policy(1, "orders", "region = '华北'");

        assert_eq!(rls.get_predicate(1, "orders"), Some("region = '华北'".to_string()));
        assert_eq!(rls.get_predicate(1, "products"), None);
        assert_eq!(rls.get_predicate(999, "orders"), None);
    }
}
