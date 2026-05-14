use crate::{ClientGtid, SyncError, SyncResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Processing,
    Committed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTransaction {
    pub cgtid: ClientGtid,
    pub status: TransactionStatus,
    pub operations_json: String,
    pub sql_statements: Option<String>,
    pub request_hash: Vec<u8>,
    pub response_blob: Option<Vec<u8>>,
    pub vector_clock_json: String,
    pub client_ts: i64,
    pub server_ts: Option<i64>,
    pub device_info: Option<String>,
}

pub trait ClientRegistry: Send + Sync {
    fn begin_transaction(&mut self, tx: &ClientTransaction) -> SyncResult<()>;
    fn update_transaction(
        &mut self,
        cgtid: &ClientGtid,
        status: TransactionStatus,
        response_blob: Option<Vec<u8>>,
        server_ts: i64,
    ) -> SyncResult<()>;
    fn get_transaction(&self, cgtid: &ClientGtid) -> SyncResult<Option<ClientTransaction>>;
    fn get_client_transactions(&self, client_id: &str) -> SyncResult<Vec<ClientTransaction>>;
    fn get_last_committed_seq(&self, client_id: &str) -> SyncResult<Option<u64>>;
    fn is_committed(&self, cgtid: &ClientGtid) -> SyncResult<bool>;
    fn is_in_progress(&self, cgtid: &ClientGtid) -> SyncResult<bool>;
    fn mark_committed(
        &mut self,
        cgtid: &ClientGtid,
        gtid: &str,
        response_blob: Option<Vec<u8>>,
        server_ts: i64,
    ) -> SyncResult<()>;
    fn mark_rolled_back(&mut self, cgtid: &ClientGtid, server_ts: i64) -> SyncResult<()>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryClientRegistry {
    transactions: HashMap<String, ClientTransaction>,
}

impl InMemoryClientRegistry {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    fn make_key(cgtid: &ClientGtid) -> String {
        cgtid.cgtid_string()
    }
}

impl ClientRegistry for InMemoryClientRegistry {
    fn begin_transaction(&mut self, tx: &ClientTransaction) -> SyncResult<()> {
        let key = Self::make_key(&tx.cgtid);
        if self.transactions.contains_key(&key) {
            return Err(SyncError::DuplicateCgtid(key));
        }
        self.transactions.insert(key, tx.clone());
        Ok(())
    }

    fn update_transaction(
        &mut self,
        cgtid: &ClientGtid,
        status: TransactionStatus,
        response_blob: Option<Vec<u8>>,
        server_ts: i64,
    ) -> SyncResult<()> {
        let key = Self::make_key(cgtid);
        if let Some(tx) = self.transactions.get_mut(&key) {
            tx.status = status;
            tx.response_blob = response_blob;
            tx.server_ts = Some(server_ts);
            Ok(())
        } else {
            Err(SyncError::RegistryError(format!(
                "Transaction not found: {}",
                key
            )))
        }
    }

    fn get_transaction(&self, cgtid: &ClientGtid) -> SyncResult<Option<ClientTransaction>> {
        let key = Self::make_key(cgtid);
        Ok(self.transactions.get(&key).cloned())
    }

    fn get_client_transactions(&self, client_id: &str) -> SyncResult<Vec<ClientTransaction>> {
        Ok(self
            .transactions
            .values()
            .filter(|tx| tx.cgtid.client_id == client_id)
            .cloned()
            .collect())
    }

    fn get_last_committed_seq(&self, client_id: &str) -> SyncResult<Option<u64>> {
        let mut max_seq = None;
        for tx in self.transactions.values() {
            if tx.cgtid.client_id == client_id
                && tx.status == TransactionStatus::Committed
                && (max_seq.is_none() || tx.cgtid.txn_seq > max_seq.unwrap())
            {
                max_seq = Some(tx.cgtid.txn_seq);
            }
        }
        Ok(max_seq)
    }

    fn is_committed(&self, cgtid: &ClientGtid) -> SyncResult<bool> {
        let key = Self::make_key(cgtid);
        Ok(self
            .transactions
            .get(&key)
            .map(|tx| tx.status == TransactionStatus::Committed)
            .unwrap_or(false))
    }

    fn is_in_progress(&self, cgtid: &ClientGtid) -> SyncResult<bool> {
        let key = Self::make_key(cgtid);
        Ok(self
            .transactions
            .get(&key)
            .map(|tx| tx.status == TransactionStatus::Processing)
            .unwrap_or(false))
    }

    fn mark_committed(
        &mut self,
        cgtid: &ClientGtid,
        _gtid: &str,
        response_blob: Option<Vec<u8>>,
        server_ts: i64,
    ) -> SyncResult<()> {
        self.update_transaction(
            cgtid,
            TransactionStatus::Committed,
            response_blob,
            server_ts,
        )
    }

    fn mark_rolled_back(&mut self, cgtid: &ClientGtid, server_ts: i64) -> SyncResult<()> {
        self.update_transaction(cgtid, TransactionStatus::RolledBack, None, server_ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_transaction(client_id: &str, txn_seq: u64) -> ClientTransaction {
        ClientTransaction {
            cgtid: ClientGtid::new(client_id, txn_seq),
            status: TransactionStatus::Processing,
            operations_json: "[]".to_string(),
            sql_statements: None,
            request_hash: vec![],
            response_blob: None,
            vector_clock_json: "{}".to_string(),
            client_ts: 0,
            server_ts: None,
            device_info: None,
        }
    }

    #[test]
    fn test_begin_transaction() {
        let mut registry = InMemoryClientRegistry::new();
        let tx = make_test_transaction("iphone-23", 1);

        registry.begin_transaction(&tx).unwrap();
        assert!(registry.is_in_progress(&tx.cgtid).unwrap());
    }

    #[test]
    fn test_duplicate_cgtid() {
        let mut registry = InMemoryClientRegistry::new();
        let tx = make_test_transaction("iphone-23", 1);

        registry.begin_transaction(&tx).unwrap();
        let result = registry.begin_transaction(&tx);

        assert!(result.is_err());
    }

    #[test]
    fn test_commit_transaction() {
        let mut registry = InMemoryClientRegistry::new();
        let tx = make_test_transaction("iphone-23", 1);

        registry.begin_transaction(&tx).unwrap();
        registry
            .mark_committed(&tx.cgtid, "server-1:100", None, 1000)
            .unwrap();

        assert!(registry.is_committed(&tx.cgtid).unwrap());
    }

    #[test]
    fn test_get_last_committed_seq() {
        let mut registry = InMemoryClientRegistry::new();

        for seq in [1, 2, 3] {
            let tx = make_test_transaction("iphone-23", seq);
            registry.begin_transaction(&tx).unwrap();
            registry
                .mark_committed(&tx.cgtid, "server-1:100", None, 1000)
                .unwrap();
        }

        assert_eq!(
            registry.get_last_committed_seq("iphone-23").unwrap(),
            Some(3)
        );
    }
}
