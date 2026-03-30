#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            ThreatSeverity::High,
            AlertType::SqlInjection,
            "Test alert".to_string(),
        );
        assert_eq!(alert.severity, ThreatSeverity::High);
        assert_eq!(alert.alert_type, AlertType::SqlInjection);
        assert!(!alert.acknowledged);
        assert!(alert.id.len() > 0);
    }

    #[test]
    fn test_alert_with_metadata() {
        let alert = Alert::new(
            ThreatSeverity::Critical,
            AlertType::SqlInjection,
            "SQL injection detected".to_string(),
        )
        .with_source_ip("192.168.1.100".to_string())
        .with_sql_pattern("SELECT * FROM users".to_string())
        .with_user("admin".to_string());

        assert_eq!(alert.source_ip, Some("192.168.1.100".to_string()));
        assert_eq!(alert.sql_pattern, Some("SELECT * FROM users".to_string()));
        assert_eq!(alert.user, Some("admin".to_string()));
    }

    #[test]
    fn test_alert_acknowledge() {
        let mut alert = Alert::new(
            ThreatSeverity::High,
            AlertType::IpBlocked,
            "IP blocked".to_string(),
        );
        assert!(!alert.acknowledged);
        alert.acknowledge();
        assert!(alert.acknowledged);
    }

    #[test]
    fn test_alert_manager_send_alert() {
        let mut manager = AlertManager::new(AlertConfig::default());
        let alert = Alert::new(
            ThreatSeverity::Critical,
            AlertType::SqlInjection,
            "Test injection".to_string(),
        );
        let result = manager.send_alert(alert);
        assert!(result.is_ok());
        assert_eq!(manager.get_alerts().len(), 1);
    }

    #[test]
    fn test_alert_manager_stats() {
        let mut manager = AlertManager::new(AlertConfig::default());
        manager
            .send_sql_injection_alert(
                Some("192.168.1.1".to_string()),
                Some("DROP TABLE".to_string()),
                "DROP TABLE attack",
            )
            .unwrap();
        manager
            .send_timeout_alert("SELECT * FROM big_table", 31)
            .unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total_alerts, 2);
        assert_eq!(
            *stats.alerts_by_type.get(&AlertType::SqlInjection).unwrap(),
            1
        );
        assert_eq!(
            *stats.alerts_by_type.get(&AlertType::QueryTimeout).unwrap(),
            1
        );
    }

    #[test]
    fn test_alert_manager_severity_filter() {
        let mut manager = AlertManager::new(AlertConfig {
            min_severity_for_alert: ThreatSeverity::High,
            ..Default::default()
        });

        let low_severity_alert = Alert::new(
            ThreatSeverity::Low,
            AlertType::BatchOperationBlocked,
            "Batch blocked".to_string(),
        );
        let result = manager.send_alert(low_severity_alert);
        assert!(result.is_ok());
        assert_eq!(manager.get_alerts().len(), 0);
    }

    #[test]
    fn test_alert_manager_disabled() {
        let mut manager = AlertManager::new(AlertConfig {
            enabled: false,
            ..Default::default()
        });

        let alert = Alert::new(
            ThreatSeverity::Critical,
            AlertType::SqlInjection,
            "Should not be sent".to_string(),
        );
        let result = manager.send_alert(alert);
        assert!(result.is_ok());
        assert_eq!(manager.get_alerts().len(), 0);
    }

    #[test]
    fn test_acknowledge_alert() {
        let mut manager = AlertManager::new(AlertConfig::default());
        let alert = Alert::new(
            ThreatSeverity::High,
            AlertType::IpBlocked,
            "IP blocked".to_string(),
        );
        manager.send_alert(alert).unwrap();

        let alert_id = manager.get_alerts()[0].id.clone();
        manager.acknowledge_alert(&alert_id).unwrap();

        assert!(manager.get_alerts()[0].acknowledged);
        assert_eq!(manager.get_stats().acknowledged_alerts, 1);
    }

    #[test]
    fn test_acknowledge_nonexistent_alert() {
        let mut manager = AlertManager::new(AlertConfig::default());
        let result = manager.acknowledge_alert("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_alerts() {
        let mut manager = AlertManager::new(AlertConfig::default());
        manager
            .send_sql_injection_alert(None, None, "test")
            .unwrap();
        manager.send_ip_blocked_alert("10.0.0.1").unwrap();
        assert_eq!(manager.get_alerts().len(), 2);

        manager.clear_alerts();
        assert_eq!(manager.get_alerts().len(), 0);
    }

    #[test]
    fn test_send_specific_alert_types() {
        let mut manager = AlertManager::new(AlertConfig::default());

        manager
            .send_full_table_scan_alert("users", "SELECT * FROM users")
            .unwrap();
        manager
            .send_batch_blocked_alert(
                "DELETE",
                "DELETE FROM users",
                Some("192.168.1.1".to_string()),
            )
            .unwrap();

        let stats = manager.get_stats();
        assert_eq!(
            *stats.alerts_by_type.get(&AlertType::FullTableScan).unwrap(),
            1
        );
        assert_eq!(
            *stats
                .alerts_by_type
                .get(&AlertType::BatchOperationBlocked)
                .unwrap(),
            1
        );
    }

    #[test]
    fn test_alert_config_default() {
        let config = AlertConfig::default();
        assert!(config.enabled);
        assert_eq!(config.buffer_size, 1000);
        assert_eq!(config.flush_interval_secs, 5);
        assert!(config.alert_on_injection);
    }

    #[test]
    fn test_queue_full() {
        let mut manager = AlertManager::new(AlertConfig {
            buffer_size: 2,
            ..Default::default()
        });

        manager
            .send_alert(Alert::new(
                ThreatSeverity::High,
                AlertType::SqlInjection,
                "1".to_string(),
            ))
            .unwrap();
        manager
            .send_alert(Alert::new(
                ThreatSeverity::High,
                AlertType::SqlInjection,
                "2".to_string(),
            ))
            .unwrap();

        let result = manager.send_alert(Alert::new(
            ThreatSeverity::High,
            AlertType::SqlInjection,
            "3".to_string(),
        ));
        assert!(matches!(result, Err(AlertError::QueueFull)));
    }
}
