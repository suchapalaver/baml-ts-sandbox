//! BAML Agent Runner
//!
//! This binary loads and executes one or more packaged agent applications.
//! Each agent package is a tar.gz containing BAML schemas, compiled TypeScript,
//! and metadata.

use baml_rt::{BamlRtError, Result, quickjs_bridge::QuickJSBridge, spans};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Agent package metadata
#[derive(Debug, Clone)]
struct AgentManifest {
    version: String,
    name: String,
    entry_point: String,
}

/// Agent package loader and executor
struct AgentPackage {
    name: String,
    js_bridge: Arc<tokio::sync::Mutex<QuickJSBridge>>,
}

impl AgentPackage {
    /// Load an agent package from a tar.gz file
    async fn load_from_file(package_path: &Path) -> Result<Self> {
        let span = spans::load_agent_package(package_path);
        let _guard = span.enter();

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

        {
            let extract_span = spans::extract_package(&extract_dir);
            let _extract_guard = extract_span.enter();

            // Extract tar.gz
            let tar_gz = std::fs::File::open(package_path)
                .map_err(|e| BamlRtError::Io(e))?;
            let tar = flate2::read::GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(tar);

            archive
                .unpack(&extract_dir)
                .map_err(|e| BamlRtError::Io(e))?;
        }

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
        {
            let schema_span = spans::load_baml_schema(&baml_src);
            let _schema_guard = schema_span.enter();
            let baml_src_str = baml_src.to_str()
                .ok_or_else(|| BamlRtError::InvalidArgument(
                    "BAML source path contains invalid UTF-8".to_string()
                ))?;
            runtime_manager.load_schema(baml_src_str)?;
            info!(agent = manifest.name, "BAML schema loaded");
        }

        // Create QuickJS bridge and expose BAML functions to it
        let runtime_manager_arc = Arc::new(Mutex::new(runtime_manager));
        let mut js_bridge = {
            let bridge_span = spans::create_js_bridge();
            let _bridge_guard = bridge_span.enter();
            let mut bridge = QuickJSBridge::new(runtime_manager_arc.clone()).await?;
            bridge.register_baml_functions().await?;
            info!(agent = manifest.name, "BAML functions registered with QuickJS");
            bridge
        };

        // Load agent's JavaScript code from dist/entry_point
        let entry_point_path = extract_dir.join(&manifest.entry_point);
        if entry_point_path.exists() {
            let eval_span = spans::evaluate_agent_code(&manifest.entry_point);
            let _eval_guard = eval_span.enter();

            let agent_code = std::fs::read_to_string(&entry_point_path)
                .map_err(|e| BamlRtError::Io(e))?;
            
            info!(entry_point = manifest.entry_point, "Loading agent JavaScript code");

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
            name: manifest.name,
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
        // Delegate to QuickJSBridge's shared implementation
        // QuickJSBridge handles checking globalThis functions and falling back to BAML
        let mut js_bridge = self.js_bridge.lock().await;
        js_bridge.invoke_function(function_name, args).await
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
        let span = spans::invoke_function(agent_name, function_name);
        let _guard = span.enter();

        let agent = self.agents.get(agent_name)
            .ok_or_else(|| BamlRtError::InvalidArgument(
                format!("Agent '{}' not found", agent_name)
            ))?;
        
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
                .add_directive("quickjs_runtime::quickjsrealmadapter=warn".parse().unwrap_or_else(|_| tracing_subscriber::filter::Directive::default()))
                .add_directive("quickjs_runtime::typescript=warn".parse().unwrap_or_else(|_| tracing_subscriber::filter::Directive::default()))
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
                    info!(package_path = %package_path.display(), "Agent package loaded");
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

