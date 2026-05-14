use crate::{ClientRegistry, OTEngine, SyncError, SyncRequest, SyncResponse, SyncResult};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait SyncEndpoint: Send + Sync {
    async fn sync(&self, request: SyncRequest) -> SyncResult<SyncResponse>;
    async fn batch_sync(&self, requests: Vec<SyncRequest>) -> SyncResult<Vec<SyncResponse>>;
}

pub struct DefaultSyncEndpoint {
    registry: Arc<Mutex<dyn ClientRegistry>>,
    ot_engine: OTEngine,
}

impl DefaultSyncEndpoint {
    pub fn new(registry: impl ClientRegistry + 'static) -> Self {
        Self {
            registry: Arc::new(Mutex::new(registry)),
            ot_engine: OTEngine::new(),
        }
    }

    pub async fn handle_sync(&mut self, request: SyncRequest) -> SyncResult<SyncResponse> {
        let cgtid = request.cgtid.clone();
        let cgtid_str = cgtid.cgtid_string();

        {
            let registry = self.registry.lock().unwrap();
            if registry.is_committed(&cgtid)? {
                let cached = registry.get_transaction(&cgtid)?;
                if let Some(tx) = cached {
                    if let Some(_blob) = tx.response_blob {
                        return Ok(SyncResponse::commit(
                            cgtid,
                            format!("cached:{}", cgtid_str),
                            tx.server_ts.unwrap_or(0),
                        ));
                    }
                }
            }

            if registry.is_in_progress(&cgtid)? {
                return Err(SyncError::CgtidInProgress(cgtid_str));
            }
        }

        self.ot_engine
            .check_dependencies(&cgtid, &cgtid.vector_clock)?;

        let client_transaction = crate::client_registry::ClientTransaction {
            cgtid: cgtid.clone(),
            status: crate::client_registry::TransactionStatus::Processing,
            operations_json: serde_json::to_string(&request.operations).unwrap_or_default(),
            sql_statements: None,
            request_hash: request.request_hash(),
            response_blob: None,
            vector_clock_json: serde_json::to_string(&cgtid.vector_clock).unwrap_or_default(),
            client_ts: request.client_timestamp.unwrap_or(0),
            server_ts: None,
            device_info: request.device_info.clone(),
        };

        {
            let mut registry = self.registry.lock().unwrap();
            registry.begin_transaction(&client_transaction)?;
        }

        let response_blob: Option<Vec<u8>> = self.execute_operations(&request.operations).await?;

        let commit_ts = chrono::Utc::now().timestamp_millis();
        let gtid = format!("sync:{}:{}", cgtid.client_id, cgtid.txn_seq);

        {
            let mut registry = self.registry.lock().unwrap();
            registry.mark_committed(&cgtid, &gtid, response_blob, commit_ts)?;
        }
        self.ot_engine.record_commit(&cgtid);

        Ok(SyncResponse::commit(cgtid, gtid, commit_ts))
    }

    async fn execute_operations(
        &self,
        _operations: &[crate::Operation],
    ) -> SyncResult<Option<Vec<u8>>> {
        Ok(Some(vec![]))
    }
}

#[async_trait]
impl SyncEndpoint for DefaultSyncEndpoint {
    async fn sync(&self, _request: SyncRequest) -> SyncResult<SyncResponse> {
        Err(SyncError::ProtocolError(
            "Use sync_with_registry instead".into(),
        ))
    }

    async fn batch_sync(&self, _requests: Vec<SyncRequest>) -> SyncResult<Vec<SyncResponse>> {
        Err(SyncError::ProtocolError(
            "Use batch_sync_with_registry instead".into(),
        ))
    }
}

impl DefaultSyncEndpoint {
    pub async fn sync_with_registry(&mut self, request: SyncRequest) -> SyncResult<SyncResponse> {
        self.handle_sync(request).await
    }

    pub async fn batch_sync_with_registry(
        &mut self,
        requests: Vec<SyncRequest>,
    ) -> SyncResult<Vec<SyncResponse>> {
        let mut responses = Vec::new();
        for request in requests {
            let response = self.handle_sync(request).await?;
            responses.push(response);
        }
        Ok(responses)
    }
}
