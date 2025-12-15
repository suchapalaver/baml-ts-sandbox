//! Tests for agent runner binary

use baml_rt::baml::BamlRuntimeManager;
use dotenvy;
use std::path::Path;
use std::fs;
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Builder;

fn create_test_agent_package(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directory for package contents
    let temp_dir = std::env::temp_dir().join(format!("test-agent-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));
    fs::create_dir_all(&temp_dir)?;

    // Create baml_src directory with a simple BAML file
    let baml_src = temp_dir.join("baml_src");
    fs::create_dir_all(&baml_src)?;
    
    // Copy existing BAML files from project if available
    let project_baml_src = Path::new("baml_src");
    if project_baml_src.exists() {
        // Copy a simple BAML file
        let simple_prompt = project_baml_src.join("simple_prompt.baml");
        if simple_prompt.exists() {
            fs::copy(&simple_prompt, baml_src.join("simple_prompt.baml"))?;
        }
    } else {
        // Create a minimal BAML file
        let baml_content = "function TestFunction(name: string) -> string {\n  client DeepSeekOpenRouter\n  prompt #\"\n    Say hello to {{ name }}.\n  \"#\n}\n\nclient DeepSeekOpenRouter {\n  provider openai-generic\n  options {\n    model \"deepseek/deepseek-chat\"\n    base_url \"https://openrouter.ai/api/v1\"\n    api_key env.OPENROUTER_API_KEY\n  }\n}\n";
        fs::write(baml_src.join("test.baml"), baml_content)?;
    }

    // Note: We no longer need baml_client - the runtime loads directly from baml_src

    // Create manifest.json
    let manifest = serde_json::json!({
        "version": "1.0.0",
        "name": "test-agent",
        "description": "Test agent package",
        "entry_point": "dist/index.js",
        "runtime_version": "0.1.0"
    });
    fs::write(temp_dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;

    // Create tar.gz
    let tar_gz = fs::File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    // Add all files from temp_dir to tar
    tar.append_dir_all(".", &temp_dir)?;
    tar.finish()?;

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
async fn test_agent_package_loading() {
    // This test verifies that we can load an agent package

    // Create a test agent package
    let package_path = std::env::temp_dir().join("test-agent-package.tar.gz");
    
    match create_test_agent_package(&package_path) {
        Ok(_) => {
            println!("Created test agent package: {}", package_path.display());
        }
        Err(e) => {
            eprintln!("Failed to create test package: {}", e);
            return;
        }
    }

    // Verify package exists
    assert!(package_path.exists(), "Test package should exist");

    // Test loading (we can't easily test the binary directly, but we can test the loading logic)
    // For now, just verify the package structure is correct
    let tar_gz = fs::File::open(&package_path).unwrap();
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    
    let extract_dir = std::env::temp_dir().join("test-agent-extract");
    fs::create_dir_all(&extract_dir).unwrap();
    archive.unpack(&extract_dir).unwrap();

    // Verify manifest exists
    let manifest_path = extract_dir.join("manifest.json");
    assert!(manifest_path.exists(), "manifest.json should exist in package");

    // Verify baml_src exists
    let baml_src = extract_dir.join("baml_src");
    assert!(baml_src.exists(), "baml_src should exist in package");

    // Clean up
    fs::remove_dir_all(&extract_dir).ok();
    fs::remove_file(&package_path).ok();
}

#[tokio::test]
async fn test_runtime_manager_loads_schema() {
    // Test that BamlRuntimeManager can load a schema
    // This is the core functionality needed for agent loading
    
    let baml_src_path = Path::new("baml_src");
    if !baml_src_path.exists() {
        println!("Skipping test: baml_src directory not found");
        return;
    }

    let mut manager = BamlRuntimeManager::new().unwrap();
    let result = manager.load_schema("baml_src");
    
    match result {
        Ok(_) => {
            assert!(manager.is_schema_loaded(), "Schema should be loaded");
        }
        Err(e) => {
            let msg = format!("Schema loading failed: {:?}", e);
            println!("{}", msg);
            // Schema loading should succeed if baml_src exists
            panic!("Schema loading failed unexpectedly: {:?}", e);
        }
    }
}
