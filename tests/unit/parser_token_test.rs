// Parser Token Tests
use sqlrustgo_parser::token::Token;

#[test]
fn test_token_variants() {
    // Test basic token variants
    let _ = Token::Select;
    let _ = Token::From;
    let _ = Token::Where;
    let _ = Token::Insert;
    let _ = Token::Update;
    let _ = Token::Delete;
    let _ = Token::Create;
    let _ = Token::Drop;
    let _ = Token::Table;
    let _ = Token::Index;
    let _ = Token::On;
    let _ = Token::And;
    let _ = Token::Or;
    let _ = Token::Not;
    let _ = Token::Equal;
    let _ = Token::Less;
    let _ = Token::Greater;
    let _ = Token::LParen;
    let _ = Token::RParen;
    let _ = Token::Comma;
    let _ = Token::Semicolon;
    let _ = Token::Star;
    let _ = Token::Plus;
    let _ = Token::Minus;
    let _ = Token::Slash;
    let _ = Token::NumberLiteral(String::new());
    let _ = Token::StringLiteral(String::new());
    let _ = Token::Identifier(String::new());
    let _ = Token::Eof;
}

#[test]
fn test_token_display() {
    assert_eq!(format!("{}", Token::Select), "SELECT");
    assert_eq!(format!("{}", Token::From), "FROM");
    assert_eq!(format!("{}", Token::Where), "WHERE");
}

#[test]
fn test_is_keyword() {
    assert!(sqlrustgo_parser::token::is_keyword("SELECT"));
    assert!(sqlrustgo_parser::token::is_keyword("FROM"));
    assert!(sqlrustgo_parser::token::is_keyword("WHERE"));
    assert!(sqlrustgo_parser::token::is_keyword("INSERT"));
    assert!(sqlrustgo_parser::token::is_keyword("UPDATE"));
    assert!(sqlrustgo_parser::token::is_keyword("DELETE"));

    assert!(!sqlrustgo_parser::token::is_keyword("not_a_keyword"));
    assert!(!sqlrustgo_parser::token::is_keyword("foo"));
}

#[test]
fn test_from_keyword() {
    assert_eq!(
        sqlrustgo_parser::token::from_keyword("SELECT"),
        Some(Token::Select)
    );
    assert_eq!(
        sqlrustgo_parser::token::from_keyword("FROM"),
        Some(Token::From)
    );
    assert_eq!(
        sqlrustgo_parser::token::from_keyword("WHERE"),
        Some(Token::Where)
    );

    assert_eq!(sqlrustgo_parser::token::from_keyword("invalid"), None);
}
