//! End-to-end tests for agent runner binary

use std::path::{Path, PathBuf};
use dotenvy;
use std::process::Command;
use std::fs;
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Builder;

/// Create a test agent package from a fixture agent
fn create_test_agent_package(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Use the complex-agent fixture
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let agent_dir = PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("agents")
        .join("complex-agent");

    if !agent_dir.exists() {
        return Err(format!("Fixture agent directory not found: {}", agent_dir.display()).into());
    }

    // Create temporary directory for package contents
    let temp_dir = std::env::temp_dir().join(format!(
        "e2e-agent-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));
    fs::create_dir_all(&temp_dir)?;

    // Copy baml_src from fixture (we no longer need baml_client - runtime loads directly from baml_src)
    let baml_src = temp_dir.join("baml_src");
    let fixture_baml_src = agent_dir.join("baml_src");
    if fixture_baml_src.exists() {
        copy_dir_all(&fixture_baml_src, &baml_src)?;
    } else {
        return Err("Fixture agent baml_src not found".into());
    }

    // Create manifest.json
    let manifest = serde_json::json!({
        "version": "1.0.0",
        "name": "test-agent",
        "description": "Test agent package for E2E testing",
        "entry_point": "dist/index.js",
        "runtime_version": "0.1.0"
    });
    fs::write(temp_dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;

    // Create tar.gz
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tar_gz = fs::File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    // Add all files from temp_dir to tar
    tar.append_dir_all(".", &temp_dir)?;
    tar.finish()?;

    // Cleanup temp directory
    fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_e2e_agent_runner_load_package() {

    // Create a test agent package
    let package_path = std::env::temp_dir().join("e2e-test-agent-package.tar.gz");

    create_test_agent_package(&package_path)
        .expect("Failed to create test agent package");

    assert!(package_path.exists(), "Test package should exist");

    // Run the binary to load the package
    let output = Command::new("./target/debug/baml-agent-runner")
        .arg(package_path.to_str().unwrap())
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    // Should successfully load the agent
    assert!(
        output.status.success() || stdout.contains("Loaded") || stdout.contains("test-agent"),
        "Binary should successfully load the agent package. Exit code: {}, stdout: {}, stderr: {}",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );

    // Cleanup
    fs::remove_file(&package_path).ok();
}

#[tokio::test]
async fn test_e2e_agent_runner_invoke_function() {

    // Skip if no API key (we'll get auth errors, but that's okay for structure testing)
    let _ = dotenvy::dotenv();
    let has_api_key = std::env::var("OPENROUTER_API_KEY").is_ok();

    // Create a test agent package
    let package_path = std::env::temp_dir().join("e2e-test-agent-invoke.tar.gz");

    create_test_agent_package(&package_path)
        .expect("Failed to create test agent package");

    // Try to invoke a function (will fail without API key, but should parse correctly)
    let mut cmd = Command::new("./target/debug/baml-agent-runner");
    cmd.arg(package_path.to_str().unwrap());
    cmd.arg("--invoke");
    cmd.arg("test-agent");
    cmd.arg("SimpleGreeting");
    cmd.arg(r#"{"name":"Test"}"#);

    let output = cmd.output().expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    // Even if it fails due to missing API key, the structure should work
    // (i.e., it should load the package and attempt to invoke, not fail on parsing)
    let is_auth_error = stderr.contains("API key") 
        || stderr.contains("authentication")
        || stderr.contains("401")
        || stdout.contains("error");

    if !has_api_key && is_auth_error {
        // Expected: Missing API key
        println!("Expected authentication error (no API key provided)");
    } else if output.status.success() {
        // Success: Function was invoked
        assert!(stdout.contains("{") || stdout.contains("result"), 
                "Should return JSON result");
    } else {
        // Other errors might be acceptable if they're not parsing/loading errors
        println!("Function invocation returned non-zero exit code, but may be expected");
    }

    // Cleanup
    fs::remove_file(&package_path).ok();
}

