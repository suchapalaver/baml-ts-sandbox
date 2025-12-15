//! Common test utilities and shared modules
//!
//! This module provides shared utilities for tests, including fixture loading and test tools.

#[path = "support/mod.rs"]
mod support_internal;

pub mod support {
    pub use super::support_internal::*;
}

pub use support::tools::*;

// Fixture helpers
use std::path::{Path, PathBuf};
use baml_rt::error::{BamlRtError, Result};

pub fn fixture_path(relative_path: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(relative_path)
}

pub fn baml_fixture(name: &str) -> PathBuf {
    fixture_path(&format!("baml/{}", name))
}

pub fn agent_fixture(name: &str) -> PathBuf {
    fixture_path(&format!("agents/{}", name))
}

pub fn package_fixture(name: &str) -> PathBuf {
    fixture_path(&format!("packages/{}", name))
}

pub fn load_baml_fixture(name: &str) -> Result<String> {
    let path = baml_fixture(name);
    std::fs::read_to_string(&path)
        .map_err(|e| BamlRtError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to load BAML fixture {}: {}", name, e)
        )))
}

pub fn fixture_exists(relative_path: &str) -> bool {
    fixture_path(relative_path).exists()
}
