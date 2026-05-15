use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpillError {
    #[error("磁盘空间不足: available={available}, required={required}")]
    OutOfDiskSpace { available: u64, required: u64 },

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("内存限制达到，无法降级: {0}")]
    FallbackFailed(String),

    #[error("分区错误: {0}")]
    PartitionError(String),
}

pub type SpillResult<T> = Result<T, SpillError>;
