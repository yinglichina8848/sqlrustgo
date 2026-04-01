use sqlrustgo_catalog::{
    AuthManager, AuthResult, ObjectRef, Privilege, ScramCredential, UserIdentity,
};

pub struct DdlExecutor<'a> {
    auth_manager: &'a mut AuthManager,
}

impl<'a> DdlExecutor<'a> {
    pub fn new(auth_manager: &'a mut AuthManager) -> Self {
        Self { auth_manager }
    }

    pub fn execute_create_user(
        &mut self,
        identities: &[UserIdentity],
        password: &str,
    ) -> AuthResult<()> {
        let credential = ScramCredential::new(password);
        let password_hash = format!(
            "SCRAM:{}:{}:{}:{}",
            credential.version,
            hex::encode(&credential.salt),
            credential.iterations,
            hex::encode(&credential.stored_key)
        );

        for identity in identities {
            self.auth_manager.create_user(identity, &password_hash)?;
        }
        Ok(())
    }

    pub fn execute_drop_user(&mut self, identities: &[UserIdentity]) -> AuthResult<()> {
        for identity in identities {
            self.auth_manager.drop_user(identity)?;
        }
        Ok(())
    }

    pub fn execute_grant(
        &mut self,
        grantee: &UserIdentity,
        privilege: Privilege,
        object: &ObjectRef,
        grantor_id: u64,
    ) -> AuthResult<()> {
        self.auth_manager.grant_privilege(
            grantee,
            privilege,
            object.object_type,
            &object.object_name,
            grantor_id,
        )?;
        Ok(())
    }

    pub fn execute_revoke(
        &mut self,
        grantee: &UserIdentity,
        privilege: Privilege,
        object: &ObjectRef,
    ) -> AuthResult<()> {
        self.auth_manager.revoke_privilege(
            grantee,
            privilege,
            object.object_type,
            &object.object_name,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_catalog::{ObjectRef, Privilege};

    #[test]
    fn test_execute_create_user() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let identities = vec![UserIdentity::new("alice", "localhost")];
        let result = executor.execute_create_user(&identities, "password123");

        assert!(result.is_ok());
        assert!(auth_manager
            .find_exact_user(&UserIdentity::new("alice", "localhost"))
            .is_some());
    }

    #[test]
    fn test_execute_create_user_multiple() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let identities = vec![
            UserIdentity::new("alice", "localhost"),
            UserIdentity::new("bob", "localhost"),
        ];
        let result = executor.execute_create_user(&identities, "password123");

        assert!(result.is_ok());
        assert!(auth_manager
            .find_exact_user(&UserIdentity::new("alice", "localhost"))
            .is_some());
        assert!(auth_manager
            .find_exact_user(&UserIdentity::new("bob", "localhost"))
            .is_some());
    }

    #[test]
    fn test_execute_drop_user() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let identities = vec![UserIdentity::new("alice", "localhost")];
        executor
            .execute_create_user(&identities, "password123")
            .unwrap();

        let result = executor.execute_drop_user(&identities);
        assert!(result.is_ok());
        assert!(auth_manager
            .find_exact_user(&UserIdentity::new("alice", "localhost"))
            .is_none());
    }

    #[test]
    fn test_execute_grant() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let grantee = UserIdentity::new("alice", "localhost");
        executor
            .execute_create_user(&[grantee.clone()], "password123")
            .unwrap();

        let object = ObjectRef::table("users");
        let result = executor.execute_grant(&grantee, Privilege::Read, &object, 0);

        assert!(result.is_ok());
        assert!(auth_manager
            .check_privilege(&grantee, &object, Privilege::Read)
            .is_ok());
    }

    #[test]
    fn test_execute_revoke() {
        let mut auth_manager = AuthManager::new();
        let grantee = UserIdentity::new("alice", "localhost");
        let object = ObjectRef::table("users");

        {
            let mut executor = DdlExecutor::new(&mut auth_manager);
            executor
                .execute_create_user(&[grantee.clone()], "password123")
                .unwrap();
            executor
                .execute_grant(&grantee, Privilege::Read, &object, 0)
                .unwrap();
        }

        assert!(auth_manager
            .check_privilege(&grantee, &object, Privilege::Read)
            .is_ok());

        {
            let mut executor = DdlExecutor::new(&mut auth_manager);
            executor
                .execute_revoke(&grantee, Privilege::Read, &object)
                .unwrap();
        }

        assert!(auth_manager
            .check_privilege(&grantee, &object, Privilege::Read)
            .is_err());
    }

    #[test]
    fn test_execute_grant_database() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let grantee = UserIdentity::new("admin", "localhost");
        executor
            .execute_create_user(&[grantee.clone()], "admin123")
            .unwrap();

        let object = ObjectRef::database("mydb");
        let result = executor.execute_grant(&grantee, Privilege::All, &object, 0);

        assert!(result.is_ok());
        assert!(auth_manager
            .check_privilege(&grantee, &object, Privilege::Read)
            .is_ok());
        assert!(auth_manager
            .check_privilege(&grantee, &object, Privilege::Insert)
            .is_ok());
    }

    #[test]
    fn test_execute_grant_column() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let grantee = UserIdentity::new("alice", "localhost");
        executor
            .execute_create_user(&[grantee.clone()], "password123")
            .unwrap();

        let object = ObjectRef::column("users", "email");
        let result = executor.execute_grant(&grantee, Privilege::Update, &object, 0);

        assert!(result.is_ok());
        assert!(auth_manager
            .check_privilege(&grantee, &object, Privilege::Update)
            .is_ok());
    }

    #[test]
    fn test_execute_create_user_with_wildcard_host() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let identities = vec![UserIdentity::new("alice", "%")];
        let result = executor.execute_create_user(&identities, "password123");

        assert!(result.is_ok());
        assert!(auth_manager
            .find_exact_user(&UserIdentity::new("alice", "%"))
            .is_some());
    }

    #[test]
    fn test_execute_drop_nonexistent_user() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let identities = vec![UserIdentity::new("nonexistent", "localhost")];
        let result = executor.execute_drop_user(&identities);

        assert!(result.is_err());
    }

    #[test]
    fn test_password_hash_format() {
        let mut auth_manager = AuthManager::new();
        let mut executor = DdlExecutor::new(&mut auth_manager);

        let identities = vec![UserIdentity::new("alice", "localhost")];
        executor
            .execute_create_user(&identities, "password123")
            .unwrap();

        let user = auth_manager
            .find_exact_user(&UserIdentity::new("alice", "localhost"))
            .unwrap();
        assert!(user.password_hash.starts_with("SCRAM:1:"));
        assert!(user.password_hash.contains(":"));
    }
}
