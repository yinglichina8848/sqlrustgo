// SQLRustGo Parser Module
pub use sqlrustgo_common::{SqlError, SqlResult};

pub mod lexer;
pub mod parser;
pub mod token;
pub mod transaction;

pub use lexer::Lexer;
pub use parser::Parser;
pub use token::Token;

pub use parser::parse;
pub use parser::{
    AggregateCall, AggregateFunction, AlterTableOperation, AlterTableStatement, BackupStatement,
    CallStatement, CheckOption, CheckStatement, ColumnDefinition, CommonTableExpression,
    CreateProcedureStatement, CreateTableStatement, CreateTriggerStatement, CreateViewStatement,
    DeleteStatement, DropIndexStatement, DropTableStatement, DropViewStatement, Expression,
    ForeignKeyRef, FtsMode, InsertStatement, JoinClause, JoinType, MergeStatement, ObjectType,
    OptimizeStatement, RepairStatement, RestoreStatement, SelectColumn, SelectStatement, Statement,
    StoredProcParam, StoredProcParamMode, StoredProcStatement, TableConstraint, TruncateStatement,
    UpdateStatement, VacuumMode, VacuumStatement, WithClause, WithSelect,
};
pub use transaction::TransactionStatement;

#[cfg(test)]
mod coverage_tests_v2 {
    include!("tests_preprocessed/coverage_increase_v2.rs");
}

#[cfg(test)]
mod coverage_tests_v3 {
    include!("tests_preprocessed/coverage_increase_v3.rs");
}

#[cfg(test)]
mod coverage_tests_v4 {
    include!("tests_preprocessed/coverage_increase_v4.rs");
}

#[cfg(test)]
mod coverage_tests_v5 {
    include!("tests_preprocessed/coverage_increase_v5.rs");
}

#[cfg(test)]
mod coverage_tests_v6 {
    include!("tests_preprocessed/coverage_increase_v6.rs");
}

#[cfg(test)]
mod coverage_tests_v7 {
    include!("tests_preprocessed/coverage_increase_v7.rs");
}
