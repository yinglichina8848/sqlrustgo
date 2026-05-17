#[cfg(test)]
mod tests {
    use sqlrustgo_gmp::correction_chain::CorrectionReason;

    #[test]
    fn test_correction_reason_creation() {
        let reason = CorrectionReason::new("DATA_ENTRY", "transcription error", "operator1");
        assert!(reason.is_ok());
    }

    #[test]
    fn test_correction_reason_empty_explanation_fails() {
        let reason = CorrectionReason::new("DATA_ENTRY", "", "operator1");
        assert!(reason.is_err());
    }
}
