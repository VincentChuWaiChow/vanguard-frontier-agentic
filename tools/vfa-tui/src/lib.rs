#![deny(warnings)]

// The library API is internal and not covered by semver guarantees (see README).
// Enabled modules remain public for integration tests and the sibling binary.
// Domain modules are always available; the default `terminal` feature preserves
// the existing CLI/TUI surface without leaking its dependencies into consumers
// that explicitly disable default features.
#[cfg(feature = "terminal")]
pub mod app;
pub mod catalog;
#[cfg(feature = "terminal")]
pub mod cli;
pub mod error;
pub mod federation;
pub mod gates;
pub mod headless;
#[cfg(feature = "terminal")]
pub mod logging;
pub mod models;
pub mod paths;
pub mod persistence;
pub mod policy;
pub mod search;
pub mod security;
pub mod subprocess;
#[cfg(feature = "terminal")]
pub mod ui;
pub mod workspace;

#[cfg(test)]
pub mod test_fixtures;
