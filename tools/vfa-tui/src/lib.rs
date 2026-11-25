#![deny(warnings)]

// The library API is internal and not covered by semver guarantees (see README).
// All modules are kept `pub`: integration/property tests (separate crates) import
// 15 of them directly, and the remaining `logging`/`paths` are consumed by the
// sibling binary crate. Narrowing them to `pub(crate)` makes their items dead
// code in the library crate, which `#![deny(warnings)]` rejects — so the public
// surface intentionally mirrors the full module set.
pub mod app;
pub mod catalog;
pub mod cli;
pub mod clock;
pub mod error;
pub mod federation;
pub mod gates;
pub mod headless;
pub mod logging;
pub mod models;
pub mod paths;
pub mod persistence;
pub mod policy;
pub mod search;
pub mod security;
pub mod subprocess;
pub mod ui;
pub mod workspace;

#[cfg(test)]
pub mod test_fixtures;
