pub use super::sync_types::{
    CommitResult, ConflictResult, ErrorResult, Operation, OperationType, ResponseResult,
    SyncRequest, SyncResponse,
};

impl SyncResponse {
    pub fn commit(cgtid: super::ClientGtid, gtid: String, commit_timestamp: i64) -> Self {
        Self {
            result: ResponseResult::Commit(CommitResult {
                cgtid,
                gtid,
                commit_timestamp,
                response_blob: None,
                updated_clock: super::VectorClock::new(),
            }),
        }
    }

    pub fn conflict(
        cgtid: super::ClientGtid,
        transformed_ops: Vec<Operation>,
        conflicts: Vec<String>,
    ) -> Self {
        Self {
            result: ResponseResult::Conflict(ConflictResult {
                cgtid,
                transformed_ops,
                conflicts,
            }),
        }
    }

    pub fn error(
        cgtid: super::ClientGtid,
        error_code: &str,
        error_message: &str,
        retryable: bool,
    ) -> Self {
        Self {
            result: ResponseResult::Error(ErrorResult {
                cgtid,
                error_code: error_code.to_string(),
                error_message: error_message.to_string(),
                retryable,
            }),
        }
    }

    pub fn is_commit(&self) -> bool {
        matches!(self.result, ResponseResult::Commit(_))
    }

    pub fn is_conflict(&self) -> bool {
        matches!(self.result, ResponseResult::Conflict(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self.result, ResponseResult::Error(_))
    }
}
