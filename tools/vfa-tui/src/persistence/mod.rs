//! Persistence module — SQLite index manager, schema migrations, single-writer task, and audit log.
//! Module skeleton — implemented in later waves.

pub mod audit;
pub mod index;
pub mod schema;
pub mod writer;

mod operation_schema;
pub mod operations;
