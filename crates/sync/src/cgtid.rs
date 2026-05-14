use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientGtid {
    pub client_id: String,
    pub txn_seq: u64,
    pub vector_clock: super::vector_clock::VectorClock,
}

impl ClientGtid {
    pub fn new(client_id: impl Into<String>, txn_seq: u64) -> Self {
        Self {
            client_id: client_id.into(),
            txn_seq,
            vector_clock: super::vector_clock::VectorClock::new(),
        }
    }

    pub fn with_clock(mut self, clock: super::vector_clock::VectorClock) -> Self {
        self.vector_clock = clock;
        self
    }

    pub fn cgtid_string(&self) -> String {
        format!("{}:{}", self.client_id, self.txn_seq)
    }

    pub fn parse(s: &str) -> Option<(String, u64)> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            let client_id = parts[0].to_string();
            let txn_seq = parts[1].parse().ok()?;
            Some((client_id, txn_seq))
        } else {
            None
        }
    }

    pub fn increment_seq(&self) -> Self {
        Self {
            client_id: self.client_id.clone(),
            txn_seq: self.txn_seq + 1,
            vector_clock: self.vector_clock.clone(),
        }
    }
}

impl fmt::Display for ClientGtid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.client_id, self.txn_seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgtid_creation() {
        let cgtid = ClientGtid::new("iphone-23", 1045);
        assert_eq!(cgtid.cgtid_string(), "iphone-23:1045");
    }

    #[test]
    fn test_cgtid_parse() {
        let (client_id, txn_seq) = ClientGtid::parse("iphone-23:1045").unwrap();
        assert_eq!(client_id, "iphone-23");
        assert_eq!(txn_seq, 1045);
    }

    #[test]
    fn test_increment_seq() {
        let cgtid = ClientGtid::new("iphone-23", 1045);
        let next = cgtid.increment_seq();
        assert_eq!(next.txn_seq, 1046);
        assert_eq!(next.client_id, "iphone-23");
    }
}
