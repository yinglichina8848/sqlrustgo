#[cfg(test)]
mod tests {
    use crate::*;

    fn create_test_firewall() -> SqlFirewall {
        SqlFirewall::new(FirewallConfig::default())
    }

    #[test]
    fn test_block_sql_injection_union() {
        let mut firewall = create_test_firewall();
        let sql = "SELECT * FROM users WHERE id=1 UNION SELECT * FROM passwords";
        let result = firewall.check_sql(sql);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FirewallError::SqlInjectionDetected(_)
        ));
    }

    #[test]
    fn test_block_sql_injection_or_classic() {
        let mut firewall = create_test_firewall();
        let sql = "SELECT * FROM users WHERE name='admin' OR '1'='1'";
        let result = firewall.check_sql(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_sql_injection_drop_table() {
        let mut firewall = create_test_firewall();
        let sql = "DROP TABLE users; --";
        let result = firewall.check_sql(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_sql_injection_exec() {
        let mut firewall = create_test_firewall();
        let sql = "EXEC sp_executesql @sql";
        let result = firewall.check_sql(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_sql_injection_file_write() {
        let mut firewall = create_test_firewall();
        let sql = "SELECT * FROM users INTO OUTFILE '/tmp/passwd'";
        let result = firewall.check_sql(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_sql_injection_comment() {
        let mut firewall = create_test_firewall();
        let sql = "SELECT * FROM users WHERE id=1 --";
        let result = firewall.check_sql(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_allow_normal_select() {
        let mut firewall = create_test_firewall();
        let sql = "SELECT id, name FROM users WHERE id = 1";
        let result = firewall.check_sql(sql);
        assert!(result.is_ok());
    }

    #[test]
    fn test_allow_normal_insert() {
        let mut firewall = create_test_firewall();
        let sql = "INSERT INTO users (name) VALUES ('test')";
        let result = firewall.check_sql(sql);
        assert!(result.is_ok());
    }

    #[test]
    fn test_whitelist_bypass() {
        let mut firewall = create_test_firewall();
        firewall.add_whitelist_pattern(WhitelistPattern {
            sql_pattern: "SELECT \\* FROM users".to_string(),
            description: "User lookup".to_string(),
            enabled: true,
        });
        let sql = "SELECT * FROM users";
        let result = firewall.check_sql(sql);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_timeout() {
        let firewall = create_test_firewall();
        let result = firewall.check_query_timeout(31);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FirewallError::QueryTimeout(_)
        ));
    }

    #[test]
    fn test_query_timeout_ok() {
        let firewall = create_test_firewall();
        let result = firewall.check_query_timeout(10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_row_limit_exceeded() {
        let firewall = create_test_firewall();
        let result = firewall.check_row_limit(10001);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FirewallError::RowLimitExceeded(_)
        ));
    }

    #[test]
    fn test_row_limit_ok() {
        let firewall = create_test_firewall();
        let result = firewall.check_row_limit(100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ip_blocking() {
        let mut firewall = create_test_firewall();
        firewall.block_ip("192.168.1.100");
        let result = firewall.check_ip("192.168.1.100");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FirewallError::IpBlocked(_)));
    }

    #[test]
    fn test_ip_not_blocked() {
        let firewall = create_test_firewall();
        let result = firewall.check_ip("192.168.1.1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_delete_blocked() {
        let firewall = create_test_firewall();
        let sql = "DELETE FROM users";
        let result = firewall.check_batch_operation(sql);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FirewallError::BatchOperationNotAllowed(_)
        ));
    }

    #[test]
    fn test_batch_update_blocked() {
        let firewall = create_test_firewall();
        let sql = "UPDATE users SET name='test'";
        let result = firewall.check_batch_operation(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_allow_single_delete() {
        let firewall = create_test_firewall();
        let sql = "DELETE FROM users WHERE id = 1";
        let result = firewall.check_batch_operation(sql);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stats_tracking() {
        let mut firewall = create_test_firewall();
        firewall.check_sql("SELECT * FROM users").unwrap();
        firewall.check_sql("SELECT * FROM users").unwrap();
        firewall.check_sql("DROP TABLE users").unwrap_err();

        let stats = firewall.get_stats();
        assert_eq!(stats.total_checked, 3);
        assert_eq!(stats.injections_blocked, 1);
    }

    #[test]
    fn test_disabled_firewall() {
        let mut firewall = SqlFirewall::new(FirewallConfig {
            enabled: false,
            ..Default::default()
        });
        let result = firewall.check_sql("DROP TABLE users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_blacklist_pattern() {
        let mut firewall = create_test_firewall();
        firewall.add_blacklist_pattern(BlacklistPattern {
            pattern: r"TABLESAMPLE".to_string(),
            description: "Sampling attack".to_string(),
            severity: ThreatSeverity::High,
        });

        let sql = "SELECT * FROM users TABLESAMPLE SYSTEM(100)";
        let result = firewall.check_sql(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_detection() {
        let mut firewall = create_test_firewall();
        let sql = "sElEcT * fRoM uSeRs WhErE iD=1 UnIoN SeLeCt * fRoM pAsSwOrDs";
        let result = firewall.check_sql(sql);
        assert!(result.is_err());
    }
}
