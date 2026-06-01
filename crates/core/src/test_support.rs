//! Shared test utilities (only compiled during `cargo test`).
use std::path::PathBuf;

/// Resolve a fixture path relative to the workspace `fixtures/` directory.
///
/// Uses `std::env::var("CARGO_MANIFEST_DIR")` (runtime) instead of
/// `env!("CARGO_MANIFEST_DIR")` (compile-time) so the path stays correct
/// even if the project directory is moved or the binary is stale.
pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
        .parent()
        .expect("CARGO_MANIFEST_DIR should have a parent")
        .parent()
        .expect("CARGO_MANIFEST_DIR grandparent should exist")
        .join("fixtures")
        .join(name)
}
