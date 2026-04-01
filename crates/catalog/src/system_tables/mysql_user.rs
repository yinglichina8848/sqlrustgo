use crate::AuthManager;

pub struct MysqlUserTable;

impl MysqlUserTable {
    pub fn schema() -> Vec<(&'static str, &'static str)> {
        vec![
            ("user", "VARCHAR(32)"),
            ("host", "VARCHAR(255)"),
            ("plugin", "VARCHAR(128)"),
            ("authentication_string", "TEXT"),
            ("account_locked", "ENUM('Y', 'N')"),
        ]
    }

    pub fn rows(auth_manager: &AuthManager) -> Vec<Vec<String>> {
        auth_manager
            .all_users()
            .map(|user| {
                vec![
                    user.identity.username.clone(),
                    user.identity.host.clone(),
                    "caching_sha2_password".to_string(),
                    encode_auth_string(&user.password_hash),
                    if user.is_active { "N" } else { "Y" }.to_string(),
                ]
            })
            .collect()
    }
}

fn encode_auth_string(password_hash: &str) -> String {
    password_hash.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthManager, UserIdentity};

    #[test]
    fn test_schema() {
        let schema = MysqlUserTable::schema();
        assert_eq!(schema.len(), 5);
        assert_eq!(schema[0], ("user", "VARCHAR(32)"));
        assert_eq!(schema[1], ("host", "VARCHAR(255)"));
        assert_eq!(schema[2], ("plugin", "VARCHAR(128)"));
        assert_eq!(schema[3], ("authentication_string", "TEXT"));
        assert_eq!(schema[4], ("account_locked", "ENUM('Y', 'N')"));
    }

    #[test]
    fn test_rows_empty() {
        let auth = AuthManager::new();
        let rows = MysqlUserTable::rows(&auth);
        assert!(rows.is_empty());
    }

    #[test]
    fn test_rows_with_users() {
        let mut auth = AuthManager::new();
        let id1 = UserIdentity::new("alice", "localhost");
        let id2 = UserIdentity::new("bob", "192.168.1.%");

        auth.create_user(&id1, "hash1").unwrap();
        auth.create_user(&id2, "hash2").unwrap();

        let rows = MysqlUserTable::rows(&auth);
        assert_eq!(rows.len(), 2);

        let alice_row = rows.iter().find(|r| r[0] == "alice").unwrap();
        assert_eq!(alice_row[1], "localhost");
        assert_eq!(alice_row[2], "caching_sha2_password");
        assert_eq!(alice_row[3], "hash1");
        assert_eq!(alice_row[4], "N");

        let bob_row = rows.iter().find(|r| r[0] == "bob").unwrap();
        assert_eq!(bob_row[1], "192.168.1.%");
    }
}
