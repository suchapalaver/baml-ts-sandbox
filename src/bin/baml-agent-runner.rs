//! BAML Agent Runner
//!
//! This binary loads and executes one or more packaged agent applications.
//! Each agent package is a tar.gz containing BAML schemas, compiled TypeScript,
//! and metadata.

use baml_rt::{BamlRtError, Result, quickjs_bridge::QuickJSBridge};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Agent package metadata
#[derive(Debug, Clone)]
struct AgentManifest {
    version: String,
    name: String,
    entry_point: String,
    description: Option<String>,
    metadata: Option<Value>,
}

/// Agent package loader and executor
struct AgentPackage {
    name: String,
    manifest: AgentManifest,
    extract_dir: PathBuf,
    runtime_manager: Arc<tokio::sync::Mutex<baml_rt::baml::BamlRuntimeManager>>,
    js_bridge: Arc<tokio::sync::Mutex<QuickJSBridge>>,
}

impl AgentPackage {
    /// Load an agent package from a tar.gz file
    async fn load_from_file(package_path: &Path) -> Result<Self> {
        info!(package = %package_path.display(), "Loading agent package");

        // Create temporary extraction directory
        let extract_dir = std::env::temp_dir().join(format!(
            "baml-agent-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));
        std::fs::create_dir_all(&extract_dir)
            .map_err(|e| BamlRtError::Io(e))?;

        info!(extract_dir = %extract_dir.display(), "Extracting agent package");

        // Extract tar.gz
        let tar_gz = std::fs::File::open(package_path)
            .map_err(|e| BamlRtError::Io(e))?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);

        archive
            .unpack(&extract_dir)
            .map_err(|e| BamlRtError::Io(e))?;

        info!("Package extracted successfully");

        // Load manifest
        let manifest_path = extract_dir.join("manifest.json");
        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| BamlRtError::Io(e))?;
        let manifest_json: Value = serde_json::from_str(&manifest_content)
            .map_err(BamlRtError::Json)?;

        let manifest = AgentManifest {
            version: manifest_json
                .get("version")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BamlRtError::InvalidArgument(
                    "manifest.json missing 'version' field".to_string()
                ))?
                .to_string(),
            name: manifest_json
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BamlRtError::InvalidArgument(
                    "manifest.json missing 'name' field".to_string()
                ))?
                .to_string(),
            entry_point: manifest_json
                .get("entry_point")
                .and_then(|v| v.as_str())
                .unwrap_or("dist/index.js")
                .to_string(),
            description: manifest_json
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            metadata: manifest_json.get("metadata").cloned(),
        };

        info!(
            name = manifest.name,
            version = manifest.version,
            entry_point = manifest.entry_point,
            "Agent manifest loaded"
        );

        // Validate package structure
        let baml_src = extract_dir.join("baml_src");
        if !baml_src.exists() {
            return Err(BamlRtError::InvalidArgument(
                "Package missing baml_src directory".to_string()
            ));
        }

        // Create runtime manager
        let mut runtime_manager = baml_rt::baml::BamlRuntimeManager::new()?;
        
        // Load BAML schema
        let baml_src_str = baml_src.to_str()
            .ok_or_else(|| BamlRtError::InvalidArgument(
                "BAML source path contains invalid UTF-8".to_string()
            ))?;
        runtime_manager.load_schema(baml_src_str)?;

        info!("BAML schema loaded for agent: {}", manifest.name);

        // Create QuickJS bridge and expose BAML functions to it
        let runtime_manager_arc = Arc::new(Mutex::new(runtime_manager));
        let mut js_bridge = QuickJSBridge::new(runtime_manager_arc.clone()).await?;
        js_bridge.register_baml_functions().await?;

        info!("BAML functions registered with QuickJS for agent: {}", manifest.name);

        // Load agent's JavaScript code from dist/entry_point
        let entry_point_path = extract_dir.join(&manifest.entry_point);
        if entry_point_path.exists() {
            let agent_code = std::fs::read_to_string(&entry_point_path)
                .map_err(|e| BamlRtError::Io(e))?;
            
            info!(
                entry_point = manifest.entry_point,
                "Loading agent JavaScript code"
            );

            // Execute the agent's code to initialize it
            // The code should expose functions that can be called later
            // We ignore the result since it's just initialization code
            match js_bridge.evaluate(&agent_code).await {
                Ok(_) => info!("Agent code executed successfully"),
                Err(e) => {
                    // Log warning but don't fail - the code might just not return a value
                    tracing::warn!(
                        error = %e,
                        "Agent code execution returned an error (may be expected)"
                    );
                }
            }

            info!("Agent JavaScript code loaded and initialized");
        } else {
            info!(
                entry_point = manifest.entry_point,
                "Agent entry point not found, skipping JavaScript initialization"
            );
        }

        Ok(Self {
            name: manifest.name.clone(),
            manifest,
            extract_dir,
            runtime_manager: runtime_manager_arc,
            js_bridge: Arc::new(Mutex::new(js_bridge)),
        })
    }

    /// Get the agent name
    fn name(&self) -> &str {
        &self.name
    }

    /// Execute a function in this agent
    /// 
    /// First tries to call it as a JavaScript function exposed by the agent,
    /// then falls back to calling it as a BAML function directly.
    async fn invoke_function(&self, function_name: &str, args: Value) -> Result<Value> {
        // Create JavaScript code to call the function
        let args_json = serde_json::to_string(&args)
            .map_err(BamlRtError::Json)?;
        
        // Try to call the function as a JavaScript function from the agent code
        // We need to parse the args_json and pass them correctly to the function
        let js_code = format!(
            r#"
            (function() {{
                try {{
                    const argsObj = {};
                    if (typeof {} === 'function') {{
                        // Call agent's JavaScript function
                        // Parse the JSON args and pass them appropriately
                        const args = {};
                        const promise = {}(args);
                        // __awaitAndStringify returns a promise - we need to handle this differently
                        // For now, return the promise and let it be handled
                        return __awaitAndStringify(promise);
                    }} else {{
                        // Fallback: call BAML function directly via runtime host
                        const promise = __baml_invoke("{}", {});
                        return __awaitAndStringify(promise);
                    }}
                }} catch (error) {{
                    return JSON.stringify({{ error: error.message || String(error) }});
                }}
            }})()
            "#,
            function_name, args_json, function_name, args_json,
            function_name, args_json
        );

        // Try calling as a JavaScript function from the agent code
        let mut js_bridge = self.js_bridge.lock().await;
        let js_result = js_bridge.evaluate(&js_code).await;
        drop(js_bridge);
        
        match js_result {
            Ok(value) => Ok(value),
            Err(_) => {
                // Fallback to direct BAML invocation
                let manager = self.runtime_manager.lock().await;
                manager.invoke_function(function_name, args).await
            }
        }
    }

    /// List available functions in this agent
    async fn list_functions(&self) -> Vec<String> {
        let manager = self.runtime_manager.lock().await;
        // TODO: Implement function listing if available
        // For now, return empty vec - functions are discovered from BAML schema
        vec![]
    }
}

/// Agent runner that manages multiple agent packages
struct AgentRunner {
    agents: HashMap<String, AgentPackage>,
}

impl AgentRunner {
    fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Load an agent package
    async fn load_agent(&mut self, package_path: &Path) -> Result<()> {
        let agent = AgentPackage::load_from_file(package_path).await?;
        let name = agent.name().to_string();
        info!(agent = name, "Agent loaded successfully");
        self.agents.insert(name.clone(), agent);
        Ok(())
    }

    /// Execute a function in a specific agent
    async fn invoke(
        &self,
        agent_name: &str,
        function_name: &str,
        args: Value,
    ) -> Result<Value> {
        let agent = self.agents.get(agent_name)
            .ok_or_else(|| BamlRtError::InvalidArgument(
                format!("Agent '{}' not found", agent_name)
            ))?;
        
        info!(
            agent = agent_name,
            function = function_name,
            "Invoking agent function"
        );
        
        agent.invoke_function(function_name, args).await
    }

    /// List all loaded agents
    fn list_agents(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("baml_rt=debug".parse().unwrap_or_default())
        )
        .init();

    info!("BAML Agent Runner starting");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <agent-package.tar.gz> [agent-package2.tar.gz ...] [--invoke <agent> <function> <json-args>]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} agent1.tar.gz agent2.tar.gz", args[0]);
        eprintln!("  {} agent1.tar.gz --invoke agent1 SimpleGreeting '{{\"name\":\"World\"}}'", args[0]);
        std::process::exit(1);
    }

    let mut runner = AgentRunner::new();

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--invoke" {
            // Invoke mode
            if i + 3 >= args.len() {
                eprintln!("Error: --invoke requires <agent> <function> <json-args>");
                std::process::exit(1);
            }
            
            let agent_name = &args[i + 1];
            let function_name = &args[i + 2];
            let json_args = &args[i + 3];
            
            let args_value: Value = serde_json::from_str(json_args)
                .map_err(|e| anyhow::anyhow!("Invalid JSON arguments: {}", e))?;
            
            let result = runner.invoke(agent_name, function_name, args_value).await
                .map_err(|e| anyhow::anyhow!("Function invocation failed: {}", e))?;
            
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        } else {
            // Load agent package
            let package_path = Path::new(&args[i]);
            if !package_path.exists() {
                eprintln!("Error: Agent package not found: {}", package_path.display());
                std::process::exit(1);
            }
            
            match runner.load_agent(package_path).await {
                Ok(_) => {
                    info!("Agent package loaded: {}", package_path.display());
                }
                Err(e) => {
                    error!(error = %e, package = %package_path.display(), "Failed to load agent package");
                    eprintln!("Error: Failed to load agent package {}: {}", package_path.display(), e);
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }

    // If we get here, just loaded agents without invoking
    let agents = runner.list_agents();
    if agents.is_empty() {
        eprintln!("Error: No agents loaded");
        std::process::exit(1);
    }

    println!("✅ Loaded {} agent(s):", agents.len());
    for agent_name in &agents {
        println!("  - {}", agent_name);
    }

    info!("Agent Runner completed successfully");
    Ok(())
}

