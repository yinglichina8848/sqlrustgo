//! XA Transaction Coordinator for MySQL 5.7 compatible distributed transactions
//!
//! Implements the XA specification for two-phase commit with MySQL syntax:
//! - `XA START 'xid'` - Begin XA transaction
//! - `XA END 'xid'` - Mark end of XA transaction
//! - `XA PREPARE 'xid'` - Prepare for commit (phase 1)
//! - `XA COMMIT 'xid'` - Commit the XA transaction (phase 2)
//! - `XA ROLLBACK 'xid'` - Rollback the XA transaction
//! - `XA RECOVER` - List prepared but not committed XA transactions
//!
//! State machine: ACTIVE → IDLE → PREPARED → COMMITTED/ROLLEDBACK

use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

/// Returns the current timestamp as seconds since Unix epoch
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// XA transaction state following MySQL 5.7 XA specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XaState {
    /// Transaction is active and can execute SQL statements
    Active,
    /// Transaction has ended; no more SQL statements allowed
    Idle,
    /// Transaction has been prepared; committed or rolled back
    Prepared,
    /// Transaction has been committed (terminal state)
    Committed,
    /// Transaction has been rolled back (terminal state)
    RolledBack,
}

impl fmt::Display for XaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XaState::Active => write!(f, "ACTIVE"),
            XaState::Idle => write!(f, "IDLE"),
            XaState::Prepared => write!(f, "PREPARED"),
            XaState::Committed => write!(f, "COMMITTED"),
            XaState::RolledBack => write!(f, "ROLLEDBACK"),
        }
    }
}

/// XA transaction identifier (XID) structure
///
/// The XID format follows the X/Open XA specification used by MySQL:
/// - `format_id`: Identifies the XA format (MySQL uses 0 for MySQL format)
/// - `gtrid`: Global transaction ID (up to 64 bytes)
/// - `bqual`: Branch qualifier (up to 64 bytes)
#[derive(Debug, Clone)]
pub struct Xid {
    /// Format identifier (0 = MySQL XA format)
    pub format_id: i32,
    /// Global transaction identifier
    pub gtrid: Vec<u8>,
    /// Branch qualifier
    pub bqual: Vec<u8>,
}

impl Xid {
    /// Creates a new XID with the given format_id, gtrid, and bqual
    pub fn new(format_id: i32, gtrid: Vec<u8>, bqual: Vec<u8>) -> Self {
        Self {
            format_id,
            gtrid,
            bqual,
        }
    }

    /// Creates an XID from a string representation (for testing)
    ///
    /// The string format is: `format_id:gtrid:bqual`
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        let format_id: i32 = parts[0].parse().ok()?;
        let gtrid = parts[1].as_bytes().to_vec();
        let bqual = parts[2].as_bytes().to_vec();
        Some(Self::new(format_id, gtrid, bqual))
    }

    /// Returns a display string for the XID
    pub fn to_display_string(&self) -> String {
        fn bytes_to_string(bytes: &[u8]) -> String {
            String::from_utf8(bytes.to_vec())
                .map(|s| s.replace('\0', "\\0"))
                .unwrap_or_else(|_| bytes.iter().map(|b| format!("{:02x}", b)).collect())
        }
        format!(
            "{}:{}:{}",
            self.format_id,
            bytes_to_string(&self.gtrid),
            bytes_to_string(&self.bqual)
        )
    }
}

impl PartialEq for Xid {
    fn eq(&self, other: &Self) -> bool {
        self.format_id == other.format_id && self.gtrid == other.gtrid && self.bqual == other.bqual
    }
}

impl Eq for Xid {}

impl Hash for Xid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.format_id.hash(state);
        self.gtrid.hash(state);
        self.bqual.hash(state);
    }
}

/// XA transaction entry stored in the coordinator
#[derive(Debug, Clone)]
pub struct XaTransaction {
    /// Transaction identifier
    pub xid: Xid,
    /// Current state of the transaction
    pub state: XaState,
    /// When the transaction was created (Unix timestamp)
    pub created_at: u64,
    /// When the transaction was last updated (Unix timestamp)
    pub updated_at: u64,
    /// Whether this transaction is a heuristic commit/rollback
    pub heuristic: bool,
    /// Heuristic outcome description if applicable
    pub heuristic_reason: Option<String>,
}

impl XaTransaction {
    /// Creates a new XA transaction
    pub fn new(xid: Xid) -> Self {
        let now = current_timestamp();
        Self {
            xid,
            state: XaState::Active,
            created_at: now,
            updated_at: now,
            heuristic: false,
            heuristic_reason: None,
        }
    }

    /// Updates the state and sets updated_at to current time
    pub fn set_state(&mut self, new_state: XaState) {
        self.state = new_state;
        self.updated_at = current_timestamp();
    }

    /// Marks the transaction as heuristically committed
    pub fn set_heuristic_commit(&mut self, reason: &str) {
        self.heuristic = true;
        self.heuristic_reason = Some(reason.to_string());
        self.set_state(XaState::Committed);
    }

    /// Marks the transaction as heuristically rolled back
    pub fn set_heuristic_rollback(&mut self, reason: &str) {
        self.heuristic = true;
        self.heuristic_reason = Some(reason.to_string());
        self.set_state(XaState::RolledBack);
    }
}

/// Errors that can occur during XA transaction operations
#[derive(Debug, Clone)]
pub enum XaError {
    /// Transaction with the given XID already exists
    XidExists(Xid),
    /// Transaction with the given XID does not exist
    XidNotFound(Xid),
    /// Invalid state transition for the XA state machine
    InvalidStateTransition {
        current_state: XaState,
        attempted_operation: String,
    },
    /// XID is already in the prepared state
    XidPrepared(Xid),
    /// Transaction is in a terminal state and cannot be modified
    XidTerminal(Xid),
    /// Malformed XID string
    MalformedXid(String),
    /// Invalid XID format
    InvalidXidFormat { expected: String, found: String },
    /// The transaction is not in a state that allows the operation
    InvalidForState(String),
}

impl fmt::Display for XaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XaError::XidExists(xid) => {
                write!(f, "XA transaction with XID {:?} already exists", xid)
            }
            XaError::XidNotFound(xid) => {
                write!(f, "XA transaction with XID {:?} not found", xid)
            }
            XaError::InvalidStateTransition {
                current_state,
                attempted_operation,
            } => {
                write!(
                    f,
                    "Invalid state transition: cannot {} when in {} state",
                    attempted_operation, current_state
                )
            }
            XaError::XidPrepared(xid) => {
                write!(f, "XA transaction with XID {:?} is already prepared", xid)
            }
            XaError::XidTerminal(xid) => {
                write!(f, "XA transaction with XID {:?} is in terminal state", xid)
            }
            XaError::MalformedXid(s) => {
                write!(f, "Malformed XID string: {}", s)
            }
            XaError::InvalidXidFormat { expected, found } => {
                write!(
                    f,
                    "Invalid XID format: expected {}, found {}",
                    expected, found
                )
            }
            XaError::InvalidForState(msg) => {
                write!(f, "Invalid operation for current state: {}", msg)
            }
        }
    }
}

impl std::error::Error for XaError {}

/// XA transaction coordinator
///
/// Manages XA transactions with MySQL 5.7 compatible state machine.
pub struct XaCoordinator {
    /// All known XA transactions
    transactions: RwLock<HashMap<Xid, XaTransaction>>,
    /// XIDs that have been prepared but not yet committed/rolled back
    /// Used for recovery purposes
    recovery_log: RwLock<Vec<Xid>>,
}

impl XaCoordinator {
    /// Creates a new XA coordinator
    pub fn new() -> Self {
        Self {
            transactions: RwLock::new(HashMap::new()),
            recovery_log: RwLock::new(Vec::new()),
        }
    }

    /// Starts a new XA transaction with the given XID
    ///
    /// Valid from: Any state (creates new transaction)
    /// State change: None (new transaction starts in Active state)
    ///
    /// # Errors
    /// - `XaError::XidExists` if a transaction with this XID already exists
    pub fn xa_start(&self, xid: Xid) -> Result<(), XaError> {
        let mut transactions = self.transactions.write().unwrap();

        if transactions.contains_key(&xid) {
            return Err(XaError::XidExists(xid));
        }

        let tx = XaTransaction::new(xid.clone());
        transactions.insert(xid, tx);

        Ok(())
    }

    /// Ends the XA transaction phase (marks end of SQL execution)
    ///
    /// Valid from: Active state
    /// State change: Active → Idle
    ///
    /// # Errors
    /// - `XaError::XidNotFound` if the transaction does not exist
    /// - `XaError::InvalidStateTransition` if not in Active state
    pub fn xa_end(&self, xid: &Xid) -> Result<(), XaError> {
        let mut transactions = self.transactions.write().unwrap();

        let tx = transactions
            .get_mut(xid)
            .ok_or_else(|| XaError::XidNotFound(xid.clone()))?;

        if tx.state != XaState::Active {
            return Err(XaError::InvalidStateTransition {
                current_state: tx.state,
                attempted_operation: "XA END".to_string(),
            });
        }

        tx.set_state(XaState::Idle);
        Ok(())
    }

    /// Prepares the XA transaction for commit (phase 1)
    ///
    /// Valid from: Idle state
    /// State change: Idle → Prepared
    ///
    /// # Errors
    /// - `XaError::XidNotFound` if the transaction does not exist
    /// - `XaError::InvalidStateTransition` if not in Idle state
    pub fn xa_prepare(&self, xid: &Xid) -> Result<(), XaError> {
        let mut transactions = self.transactions.write().unwrap();

        let tx = transactions
            .get_mut(xid)
            .ok_or_else(|| XaError::XidNotFound(xid.clone()))?;

        if tx.state != XaState::Idle {
            return Err(XaError::InvalidStateTransition {
                current_state: tx.state,
                attempted_operation: "XA PREPARE".to_string(),
            });
        }

        tx.set_state(XaState::Prepared);

        drop(transactions);
        self.add_to_recovery_log(xid);

        Ok(())
    }

    /// Commits the XA transaction (phase 2)
    ///
    /// Valid from: Prepared state
    /// State change: Prepared → Committed (terminal)
    ///
    /// # Errors
    /// - `XaError::XidNotFound` if the transaction does not exist
    /// - `XaError::InvalidStateTransition` if not in Prepared state
    pub fn xa_commit(&self, xid: &Xid) -> Result<(), XaError> {
        let mut transactions = self.transactions.write().unwrap();

        let tx = transactions
            .get_mut(xid)
            .ok_or_else(|| XaError::XidNotFound(xid.clone()))?;

        if tx.state != XaState::Prepared {
            return Err(XaError::InvalidStateTransition {
                current_state: tx.state,
                attempted_operation: "XA COMMIT".to_string(),
            });
        }

        tx.set_state(XaState::Committed);

        drop(transactions);
        self.remove_from_recovery_log(xid);

        Ok(())
    }

    /// Rolls back the XA transaction
    ///
    /// Valid from: Active, Idle, or Prepared states
    /// State change: Any non-terminal → RolledBack (terminal)
    ///
    /// # Errors
    /// - `XaError::XidNotFound` if the transaction does not exist
    /// - `XaError::XidTerminal` if transaction is already committed
    pub fn xa_rollback(&self, xid: &Xid) -> Result<(), XaError> {
        let mut transactions = self.transactions.write().unwrap();

        let tx = transactions
            .get_mut(xid)
            .ok_or_else(|| XaError::XidNotFound(xid.clone()))?;

        match tx.state {
            XaState::Committed | XaState::RolledBack => {
                return Err(XaError::XidTerminal(xid.clone()));
            }
            XaState::Active | XaState::Idle | XaState::Prepared => {
                tx.set_state(XaState::RolledBack);
            }
        }

        drop(transactions);
        self.remove_from_recovery_log(xid);

        Ok(())
    }

    /// Returns all prepared XA transactions for recovery purposes
    ///
    /// This corresponds to MySQL's `XA RECOVER` command which lists
    /// all XA transactions that are in the prepared state.
    ///
    /// # Errors
    /// Returns an error if the recovery log cannot be read
    pub fn xa_recover(&self) -> Result<Vec<Xid>, XaError> {
        let recovery_log = self.recovery_log.read().unwrap();
        Ok(recovery_log.clone())
    }

    /// Returns the state of an XA transaction
    ///
    /// # Errors
    /// - `XaError::XidNotFound` if the transaction does not exist
    pub fn get_state(&self, xid: &Xid) -> Result<XaState, XaError> {
        let transactions = self.transactions.read().unwrap();
        let tx = transactions
            .get(xid)
            .ok_or_else(|| XaError::XidNotFound(xid.clone()))?;
        Ok(tx.state)
    }

    /// Checks if an XA transaction exists
    pub fn contains(&self, xid: &Xid) -> bool {
        let transactions = self.transactions.read().unwrap();
        transactions.contains_key(xid)
    }

    /// Returns the number of active (non-terminal) transactions
    pub fn num_active_transactions(&self) -> usize {
        let transactions = self.transactions.read().unwrap();
        transactions
            .values()
            .filter(|tx| tx.state != XaState::Committed && tx.state != XaState::RolledBack)
            .count()
    }

    /// Returns the number of prepared transactions (for recovery)
    pub fn num_prepared_transactions(&self) -> usize {
        let transactions = self.transactions.read().unwrap();
        transactions
            .values()
            .filter(|tx| tx.state == XaState::Prepared)
            .count()
    }

    /// Cleans up completed (terminal) transactions
    ///
    /// Removes transactions that are in Committed or RolledBack state
    pub fn cleanup_completed(&self) {
        let mut transactions = self.transactions.write().unwrap();
        transactions
            .retain(|_, tx| tx.state != XaState::Committed && tx.state != XaState::RolledBack);
    }

    /// Returns all transactions in the coordinator
    pub fn get_all_transactions(&self) -> Vec<XaTransaction> {
        let transactions = self.transactions.read().unwrap();
        transactions.values().cloned().collect()
    }

    /// Adds an XID to the recovery log
    fn add_to_recovery_log(&self, xid: &Xid) {
        let mut recovery_log = self.recovery_log.write().unwrap();
        if !recovery_log.contains(xid) {
            recovery_log.push(xid.clone());
        }
    }

    /// Removes an XID from the recovery log
    fn remove_from_recovery_log(&self, xid: &Xid) {
        let mut recovery_log = self.recovery_log.write().unwrap();
        recovery_log.retain(|x| x != xid);
    }
}

impl Default for XaCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_xid() -> Xid {
        Xid::new(0, b"gtrid1".to_vec(), b"bqual1".to_vec())
    }

    #[test]
    fn test_xid_creation() {
        let xid = Xid::new(0, b"test_gtrid".to_vec(), b"test_bqual".to_vec());
        assert_eq!(xid.format_id, 0);
        assert_eq!(xid.gtrid, b"test_gtrid");
        assert_eq!(xid.bqual, b"test_bqual");
    }

    #[test]
    fn test_xid_from_string() {
        let xid = Xid::from_string("0:gar1:bar1").unwrap();
        assert_eq!(xid.format_id, 0);
        assert_eq!(xid.gtrid, b"gar1");
        assert_eq!(xid.bqual, b"bar1");
    }

    #[test]
    fn test_xid_from_string_invalid() {
        assert!(Xid::from_string("invalid").is_none());
        assert!(Xid::from_string("0:1").is_none());
    }

    #[test]
    fn test_xid_display_string() {
        let xid = Xid::new(0, b"abc".to_vec(), b"def".to_vec());
        let s = xid.to_display_string();
        assert!(s.contains("abc"));
    }

    #[test]
    fn test_xid_partial_eq() {
        let xid1 = create_test_xid();
        let xid2 = Xid::new(0, b"gtrid1".to_vec(), b"bqual1".to_vec());
        let xid3 = Xid::new(1, b"gtrid1".to_vec(), b"bqual1".to_vec());

        assert_eq!(xid1, xid2);
        assert_ne!(xid1, xid3);
    }

    #[test]
    fn test_xid_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;

        let xid1 = create_test_xid();
        let xid2 = Xid::new(0, b"gtrid1".to_vec(), b"bqual1".to_vec());
        let xid3 = Xid::new(0, b"gtrid2".to_vec(), b"bqual1".to_vec());

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        xid1.hash(&mut hasher1);
        xid2.hash(&mut hasher2);
        xid3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    #[test]
    fn test_xa_state_display() {
        assert_eq!(format!("{}", XaState::Active), "ACTIVE");
        assert_eq!(format!("{}", XaState::Idle), "IDLE");
        assert_eq!(format!("{}", XaState::Prepared), "PREPARED");
        assert_eq!(format!("{}", XaState::Committed), "COMMITTED");
        assert_eq!(format!("{}", XaState::RolledBack), "ROLLEDBACK");
    }

    #[test]
    fn test_xa_transaction_new() {
        let xid = create_test_xid();
        let tx = XaTransaction::new(xid.clone());

        assert_eq!(tx.xid, xid);
        assert_eq!(tx.state, XaState::Active);
        assert!(!tx.heuristic);
        assert!(tx.heuristic_reason.is_none());
    }

    #[test]
    fn test_xa_transaction_set_state() {
        let tx = XaTransaction::new(create_test_xid());
        assert_eq!(tx.state, XaState::Active);

        let mut tx = tx;
        tx.set_state(XaState::Idle);
        assert_eq!(tx.state, XaState::Idle);
    }

    #[test]
    fn test_xa_transaction_heuristic() {
        let mut tx = XaTransaction::new(create_test_xid());

        tx.set_heuristic_commit("network failure");
        assert!(tx.heuristic);
        assert_eq!(tx.heuristic_reason, Some("network failure".to_string()));
        assert_eq!(tx.state, XaState::Committed);

        let mut tx2 = XaTransaction::new(create_test_xid());
        tx2.set_heuristic_rollback("disk full");
        assert!(tx2.heuristic);
        assert_eq!(tx2.heuristic_reason, Some("disk full".to_string()));
        assert_eq!(tx2.state, XaState::RolledBack);
    }

    #[test]
    fn test_xa_coordinator_new() {
        let coordinator = XaCoordinator::new();
        assert_eq!(coordinator.num_active_transactions(), 0);
    }

    #[test]
    fn test_xa_start() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        assert!(coordinator.xa_start(xid.clone()).is_ok());
        assert!(coordinator.contains(&xid));
        assert_eq!(coordinator.num_active_transactions(), 1);
    }

    #[test]
    fn test_xa_start_duplicate() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        let result = coordinator.xa_start(xid.clone());

        assert!(result.is_err());
        match result {
            Err(XaError::XidExists(_)) => {}
            _ => panic!("Expected XidExists error"),
        }
    }

    #[test]
    fn test_xa_end() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Active);

        coordinator.xa_end(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Idle);
    }

    #[test]
    fn test_xa_end_invalid_state() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();

        let result = coordinator.xa_end(&xid);
        assert!(result.is_err());
        match result {
            Err(XaError::InvalidStateTransition {
                current_state: XaState::Idle,
                ..
            }) => {}
            _ => panic!("Expected InvalidStateTransition error"),
        }
    }

    #[test]
    fn test_xa_end_not_found() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        let result = coordinator.xa_end(&xid);
        assert!(result.is_err());
        match result {
            Err(XaError::XidNotFound(_)) => {}
            _ => panic!("Expected XidNotFound error"),
        }
    }

    #[test]
    fn test_xa_prepare() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Idle);

        coordinator.xa_prepare(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Prepared);
    }

    #[test]
    fn test_xa_prepare_invalid_state() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();

        let result = coordinator.xa_prepare(&xid);
        assert!(result.is_err());
        match result {
            Err(XaError::InvalidStateTransition {
                current_state: XaState::Active,
                ..
            }) => {}
            _ => panic!("Expected InvalidStateTransition error"),
        }
    }

    #[test]
    fn test_xa_commit() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        coordinator.xa_prepare(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Prepared);

        coordinator.xa_commit(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Committed);
    }

    #[test]
    fn test_xa_commit_invalid_state() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();

        let result = coordinator.xa_commit(&xid);
        assert!(result.is_err());
        match result {
            Err(XaError::InvalidStateTransition {
                current_state: XaState::Active,
                ..
            }) => {}
            _ => panic!("Expected InvalidStateTransition error"),
        }
    }

    #[test]
    fn test_xa_rollback_active() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Active);

        coordinator.xa_rollback(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::RolledBack);
    }

    #[test]
    fn test_xa_rollback_idle() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Idle);

        coordinator.xa_rollback(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::RolledBack);
    }

    #[test]
    fn test_xa_rollback_prepared() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        coordinator.xa_prepare(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Prepared);

        coordinator.xa_rollback(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::RolledBack);
    }

    #[test]
    fn test_xa_rollback_committed_fails() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        coordinator.xa_prepare(&xid).unwrap();
        coordinator.xa_commit(&xid).unwrap();

        let result = coordinator.xa_rollback(&xid);
        assert!(result.is_err());
        match result {
            Err(XaError::XidTerminal(_)) => {}
            _ => panic!("Expected XidTerminal error"),
        }
    }

    #[test]
    fn test_xa_recover_empty() {
        let coordinator = XaCoordinator::new();
        let recovered = coordinator.xa_recover().unwrap();
        assert!(recovered.is_empty());
    }

    #[test]
    fn test_xa_recover_after_prepare() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        coordinator.xa_prepare(&xid).unwrap();

        let recovered = coordinator.xa_recover().unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0], xid);
    }

    #[test]
    fn test_xa_recover_after_commit() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        coordinator.xa_prepare(&xid).unwrap();
        coordinator.xa_commit(&xid).unwrap();

        let recovered = coordinator.xa_recover().unwrap();
        assert!(recovered.is_empty());
    }

    #[test]
    fn test_full_xa_transaction_cycle() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Active);

        coordinator.xa_end(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Idle);

        coordinator.xa_prepare(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Prepared);

        coordinator.xa_commit(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Committed);

        assert_eq!(coordinator.num_prepared_transactions(), 0);
        assert_eq!(coordinator.num_active_transactions(), 0);
    }

    #[test]
    fn test_full_xa_transaction_rollback_cycle() {
        let coordinator = XaCoordinator::new();
        let xid = create_test_xid();

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        coordinator.xa_prepare(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::Prepared);

        coordinator.xa_rollback(&xid).unwrap();
        assert_eq!(coordinator.get_state(&xid).unwrap(), XaState::RolledBack);
    }

    #[test]
    fn test_cleanup_completed() {
        let coordinator = XaCoordinator::new();
        let xid1 = Xid::new(0, b"gtrid1".to_vec(), b"bqual1".to_vec());
        let xid2 = Xid::new(0, b"gtrid2".to_vec(), b"bqual2".to_vec());

        coordinator.xa_start(xid1.clone()).unwrap();
        coordinator.xa_end(&xid1).unwrap();
        coordinator.xa_prepare(&xid1).unwrap();
        coordinator.xa_commit(&xid1).unwrap();

        coordinator.xa_start(xid2.clone()).unwrap();
        coordinator.xa_end(&xid2).unwrap();
        coordinator.xa_prepare(&xid2).unwrap();

        assert_eq!(coordinator.num_active_transactions(), 1);

        coordinator.cleanup_completed();

        assert_eq!(coordinator.num_active_transactions(), 1);
        assert!(!coordinator.contains(&xid1));
        assert!(coordinator.contains(&xid2));
    }

    #[test]
    fn test_multiple_transactions() {
        let coordinator = XaCoordinator::new();
        let xids: Vec<Xid> = (0..5)
            .map(|i| Xid::new(0, format!("gtrid{}", i).into_bytes(), b"bqual".to_vec()))
            .collect();

        for xid in &xids {
            coordinator.xa_start(xid.clone()).unwrap();
        }
        assert_eq!(coordinator.num_active_transactions(), 5);

        for xid in &xids {
            coordinator.xa_end(xid).unwrap();
        }

        for xid in &xids {
            coordinator.xa_prepare(xid).unwrap();
        }

        assert_eq!(coordinator.xa_recover().unwrap().len(), 5);

        for xid in &xids {
            coordinator.xa_commit(xid).unwrap();
        }

        assert!(coordinator.xa_recover().unwrap().is_empty());
    }

    #[test]
    fn test_get_all_transactions() {
        let coordinator = XaCoordinator::new();
        let xid1 = create_test_xid();
        let xid2 = Xid::new(0, b"gtrid2".to_vec(), b"bqual2".to_vec());

        coordinator.xa_start(xid1.clone()).unwrap();
        coordinator.xa_start(xid2.clone()).unwrap();

        let all = coordinator.get_all_transactions();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_xa_error_display() {
        let xid = create_test_xid();
        let err = XaError::XidNotFound(xid.clone());
        assert!(err.to_string().contains("not found"));

        let err2 = XaError::InvalidStateTransition {
            current_state: XaState::Active,
            attempted_operation: "XA COMMIT".to_string(),
        };
        assert!(err2.to_string().contains("ACTIVE"));
        assert!(err2.to_string().contains("XA COMMIT"));

        let err3 = XaError::MalformedXid("bad".to_string());
        assert!(err3.to_string().contains("Malformed"));
    }
}
