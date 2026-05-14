//! L2 Integration Test: Parser → AST for BEGIN IDEMPOTENT

#[cfg(test)]
mod tests {
    use sqlrustgo_parser::*;

    #[test]
    fn test_parse_begin_idempotent_string_key() {
        let sql = "BEGIN IDEMPOTENT 'txn-123'";
        let ast = parse(sql).unwrap();

        match ast {
            Statement::Transaction(TransactionStatement::BeginIdempotent { key }) => {
                assert_eq!(key, "txn-123");
            }
            _ => panic!("Expected BeginIdempotent, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_begin_idempotent_keyword_form() {
        let sql = "BEGIN IDEMPOTENCY KEY 'txn-456'";
        let ast = parse(sql).unwrap();

        match ast {
            Statement::Transaction(TransactionStatement::BeginIdempotent { key }) => {
                assert_eq!(key, "txn-456");
            }
            _ => panic!("Expected BeginIdempotent, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_begin_idempotent_identifier_key() {
        let sql = "BEGIN IDEMPOTENT my_transaction_key";
        let ast = parse(sql).unwrap();

        match ast {
            Statement::Transaction(TransactionStatement::BeginIdempotent { key }) => {
                assert_eq!(key, "my_transaction_key");
            }
            _ => panic!("Expected BeginIdempotent, got {:?}", ast),
        }
    }
}
