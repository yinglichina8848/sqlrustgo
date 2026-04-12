#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    fn get_doc_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("docs/monitoring");
        path.push(filename);
        path
    }

    #[test]
    fn test_grafana_dashboard_json_exists() {
        let path = get_doc_path("grafana-dashboard.json");
        assert!(path.exists(), "Grafana dashboard JSON should exist");
    }

    #[test]
    fn test_grafana_dashboard_json_valid() {
        let path = get_doc_path("grafana-dashboard.json");
        let content = fs::read_to_string(&path).expect("Should read dashboard file");

        // Verify it's valid JSON
        serde_json::from_str::<serde_json::Value>(&content)
            .expect("Dashboard JSON should be valid");
    }

    #[test]
    fn test_grafana_dashboard_has_required_fields() {
        let path = get_doc_path("grafana-dashboard.json");
        let content = fs::read_to_string(&path).expect("Should read dashboard file");
        let json: serde_json::Value = serde_json::from_str(&content).expect("Should parse JSON");

        // Check required fields
        assert!(json.get("title").is_some(), "Dashboard should have title");
        assert!(json.get("panels").is_some(), "Dashboard should have panels");
        assert!(
            json.get("schemaVersion").is_some(),
            "Dashboard should have schemaVersion"
        );
        assert!(json.get("tags").is_some(), "Dashboard should have tags");
    }

    #[test]
    fn test_grafana_dashboard_has_sqlrustgo_tag() {
        let path = get_doc_path("grafana-dashboard.json");
        let content = fs::read_to_string(&path).expect("Should read dashboard file");
        let json: serde_json::Value = serde_json::from_str(&content).expect("Should parse JSON");

        let tags = json.get("tags").expect("Should have tags");
        let tags_array = tags.as_array().expect("Tags should be an array");
        let has_sqlrustgo = tags_array.iter().any(|t| t.as_str() == Some("sqlrustgo"));
        assert!(has_sqlrustgo, "Dashboard should have sqlrustgo tag");
    }

    #[test]
    fn test_prometheus_alerts_yml_exists() {
        let path = get_doc_path("prometheus-alerts.yml");
        assert!(path.exists(), "Prometheus alerts YAML should exist");
    }

    #[test]
    fn test_prometheus_alerts_yaml_valid() {
        let path = get_doc_path("prometheus-alerts.yml");
        let content = fs::read_to_string(&path).expect("Should read alerts file");

        // Verify it's valid YAML
        serde_yaml::from_str::<serde_yaml::Value>(&content).expect("Alerts YAML should be valid");
    }

    #[test]
    fn test_prometheus_alerts_has_groups() {
        let path = get_doc_path("prometheus-alerts.yml");
        let content = fs::read_to_string(&path).expect("Should read alerts file");
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content).expect("Should parse YAML");

        assert!(yaml.get("groups").is_some(), "Alerts should have groups");
    }

    #[test]
    fn test_prometheus_alerts_structure() {
        let path = get_doc_path("prometheus-alerts.yml");
        let content = fs::read_to_string(&path).expect("Should read alerts file");
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content).expect("Should parse YAML");

        // Verify structure: groups[].rules[].alert
        let groups = yaml
            .get("groups")
            .expect("Should have groups")
            .as_sequence()
            .expect("Groups should be a list");

        let mut alert_count = 0;
        for group in groups {
            if let Some(rules) = group.get("rules").and_then(|v| v.as_sequence()) {
                for rule in rules {
                    if rule.get("alert").is_some() {
                        alert_count += 1;
                    }
                }
            }
        }

        assert!(alert_count >= 5, "Should have at least 5 alert rules");
    }
}
