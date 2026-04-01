use crate::{AuthManager, Privilege};

pub struct MysqlDbTable;

impl MysqlDbTable {
    pub fn schema() -> Vec<(&'static str, &'static str)> {
        vec![
            ("host", "VARCHAR(255)"),
            ("db", "VARCHAR(64)"),
            ("user", "VARCHAR(32)"),
            ("select_priv", "ENUM('Y', 'N')"),
            ("insert_priv", "ENUM('Y', 'N')"),
            ("update_priv", "ENUM('Y', 'N')"),
            ("delete_priv", "ENUM('Y', 'N')"),
            ("create_priv", "ENUM('Y', 'N')"),
            ("drop_priv", "ENUM('Y', 'N')"),
            ("grant_priv", "ENUM('Y', 'N')"),
        ]
    }

    pub fn rows(auth_manager: &AuthManager) -> Vec<Vec<String>> {
        let mut rows = Vec::new();

        for (identity, grants) in auth_manager.all_privileges() {
            for grant in grants {
                if let Some(db_name) = extract_db_name(&grant.object_name) {
                    rows.push(vec![
                        identity.host.clone(),
                        db_name,
                        identity.username.clone(),
                        bool_to_yn(grant.privilege == Privilege::Read),
                        bool_to_yn(grant.privilege == Privilege::Insert),
                        bool_to_yn(grant.privilege == Privilege::Update),
                        bool_to_yn(grant.privilege == Privilege::Delete),
                        bool_to_yn(grant.privilege == Privilege::Create),
                        bool_to_yn(grant.privilege == Privilege::Drop),
                        bool_to_yn(grant.privilege == Privilege::Grant),
                    ]);
                }
            }
        }

        rows
    }
}

fn extract_db_name(object_name: &str) -> Option<String> {
    object_name.split('.').next().map(String::from)
}

fn bool_to_yn(value: bool) -> String {
    if value { "Y" } else { "N" }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AuthManager, ObjectType, UserIdentity};

    #[test]
    fn test_schema() {
        let schema = MysqlDbTable::schema();
        assert_eq!(schema.len(), 10);
        assert_eq!(schema[0], ("host", "VARCHAR(255)"));
        assert_eq!(schema[1], ("db", "VARCHAR(64)"));
        assert_eq!(schema[2], ("user", "VARCHAR(32)"));
        assert_eq!(schema[3], ("select_priv", "ENUM('Y', 'N')"));
        assert_eq!(schema[9], ("grant_priv", "ENUM('Y', 'N')"));
    }

    #[test]
    fn test_rows_empty() {
        let auth = AuthManager::new();
        let rows = MysqlDbTable::rows(&auth);
        assert!(rows.is_empty());
    }

    #[test]
    fn test_rows_with_privileges() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");

        auth.create_user(&identity, "hash").unwrap();
        auth.grant_privilege(&identity, Privilege::Read, ObjectType::Database, "mydb", 0)
            .unwrap();
        auth.grant_privilege(
            &identity,
            Privilege::Insert,
            ObjectType::Database,
            "mydb",
            0,
        )
        .unwrap();

        let rows = MysqlDbTable::rows(&auth);
        assert_eq!(rows.len(), 2);

        let read_row = rows.iter().find(|r| r[3] == "Y").unwrap();
        assert_eq!(read_row[0], "localhost");
        assert_eq!(read_row[1], "mydb");
        assert_eq!(read_row[2], "alice");
        assert_eq!(read_row[4], "N");

        let insert_row = rows.iter().find(|r| r[4] == "Y").unwrap();
        assert_eq!(insert_row[3], "N");
        assert_eq!(insert_row[4], "Y");
    }

    #[test]
    fn test_extract_db_name() {
        assert_eq!(extract_db_name("mydb"), Some("mydb".to_string()));
        assert_eq!(extract_db_name("mydb.table"), Some("mydb".to_string()));
        assert_eq!(extract_db_name(""), Some("".to_string()));
    }

    #[test]
    fn test_bool_to_yn() {
        assert_eq!(bool_to_yn(true), "Y");
        assert_eq!(bool_to_yn(false), "N");
    }
}
