//! BAML Agent Builder
//!
//! This binary compiles, lints, and packages BAML + TypeScript agent applications
//! into distributable tar.gz packages, and runs agents with stdin/stdout connectivity.
//!
//! Uses OXC for high-performance TypeScript compilation and linting.

use baml_rt::{BamlRtError, Result};
use baml_rt::builder::{
    AgentDir, PackagePath, FunctionName, BuildDir,
    BuilderService, StdFileSystem, OxcLinter,
    OxcTypeScriptCompiler, RuntimeTypeGenerator, StdPackager,
    FileSystem, Linter,
};
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::fs;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "baml-agent-builder")]
#[command(about = "Build and run BAML agent packages with OXC", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint TypeScript/JavaScript source code
    Lint {
        /// Agent directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        agent_dir: PathBuf,
    },
    
    /// Package an agent into a tar.gz file
    Package {
        /// Agent directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        agent_dir: PathBuf,

        /// Output file path
        #[arg(short, long, default_value = "agent-package.tar.gz")]
        output: PathBuf,

        /// Skip linting
        #[arg(long)]
        skip_lint: bool,
    },
    
    /// Run an agent package with stdin/stdout connectivity
    Run {
        /// Agent package file path
        #[arg(short, long)]
        package: PathBuf,

        /// Function to call (if not provided, reads from stdin)
        #[arg(short, long)]
        function: Option<String>,

        /// JSON arguments (if not provided and function specified, reads from stdin)
        #[arg(short, long)]
        args: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Lint { agent_dir } => {
            let agent_dir = AgentDir::new(agent_dir)?;
            lint_agent(&agent_dir).await?;
        }
        Commands::Package {
            agent_dir,
            output,
            skip_lint,
        } => {
            let agent_dir = AgentDir::new(agent_dir)?;
            package_agent(&agent_dir, &output, !skip_lint).await?;
        }
        Commands::Run {
            package,
            function,
            args,
        } => {
            let package_path = PackagePath::new(package)?;
            let function_name = function
                .map(|s| FunctionName::new(s))
                .transpose()?;
            run_agent(&package_path, function_name.as_ref(), args.as_deref()).await?;
        }
    }

    Ok(())
}

async fn lint_agent(agent_dir: &AgentDir) -> Result<()> {
    tracing::info!(agent_dir = %agent_dir, "Linting agent");
    
    let filesystem = StdFileSystem;
    let linter = OxcLinter::new(filesystem);
    linter.lint(agent_dir).await
}

async fn package_agent(
    agent_dir: &AgentDir,
    output: &std::path::Path,
    lint: bool,
) -> Result<()> {
    tracing::info!(
        agent_dir = %agent_dir,
        output = %output.display(),
        "Building agent package"
    );

    println!("📦 Building agent package...");
    println!("   Agent directory: {}", agent_dir);
    println!("   Output: {}", output.display());

    // Create temporary build directory
    let build_dir = BuildDir::new()?;

    // Initialize services
    let filesystem = StdFileSystem;
    let linter = OxcLinter::new(filesystem.clone());
    let ts_compiler = OxcTypeScriptCompiler::new(filesystem.clone());
    let type_generator = RuntimeTypeGenerator::new();
    let packager = StdPackager::new(filesystem.clone());

    // Copy baml_src to build directory (runtime loads from baml_src)
    filesystem.copy_dir_all(&agent_dir.baml_src(), &build_dir.join("baml_src"))?;

    let builder_service = BuilderService::new(
        linter,
        ts_compiler,
        type_generator,
        packager,
    );

    // Build the package
    builder_service.build_package(agent_dir, &build_dir, output, lint).await?;

    println!("\n✅ Agent package built successfully: {}", output.display());
    Ok(())
}

async fn run_agent(
    package_path: &PackagePath,
    function: Option<&FunctionName>,
    args_json: Option<&str>,
) -> Result<()> {
    tracing::info!(package = %package_path, "Running agent");

    // Load the agent package
    println!("📦 Loading agent package: {}", package_path);
    let agent = load_agent_package(package_path.as_path()).await?;
    println!("✅ Agent loaded: {}", agent.name());

    // If function is specified, call it once
    if let Some(function_name) = function {
        let args = if let Some(args_str) = args_json {
            serde_json::from_str(args_str)
                .map_err(|e| BamlRtError::InvalidArgument(format!("Invalid JSON args: {}", e)))?
        } else {
            // Read args from stdin
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| BamlRtError::Io(e))?;
            serde_json::from_str(input.trim())
                .map_err(|e| BamlRtError::InvalidArgument(format!("Invalid JSON from stdin: {}", e)))?
        };

        let result = agent.invoke_function(function_name.as_str(), args).await?;
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    // Otherwise, run in interactive mode: read from stdin, write to stdout
    println!("🔄 Running in interactive mode (reading from stdin, writing to stdout)");
    println!("   Format: <function_name> <json_args>");
    println!(r#"   Example: greetUser {{"name": "Alice"}}"#);
    println!("   Press Ctrl+D to exit\n");

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        print!("> ");
        io::stdout().flush().map_err(BamlRtError::Io)?;

        match stdin_lock.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Parse input: function_name json_args
                let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
                if parts.len() < 2 {
                    eprintln!("Error: Expected format: <function_name> <json_args>");
                    continue;
                }

                let function_name_str = parts[0];
                let args_json = parts[1];

                match serde_json::from_str::<Value>(args_json) {
                    Ok(args) => {
                        match agent.invoke_function(function_name_str, args).await {
                            Ok(result) => {
                                println!("{}", serde_json::to_string_pretty(&result)?);
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: Invalid JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                return Err(BamlRtError::Io(e));
            }
        }
    }

    println!("\n👋 Exiting");
    Ok(())
}

// Agent package loader (reusing logic from baml-agent-runner)
struct LoadedAgent {
    name: String,
    runtime_manager: Arc<tokio::sync::Mutex<baml_rt::baml::BamlRuntimeManager>>,
    js_bridge: Arc<tokio::sync::Mutex<baml_rt::quickjs_bridge::QuickJSBridge>>,
}

impl LoadedAgent {
    fn name(&self) -> &str {
        &self.name
    }

    async fn invoke_function(&self, function_name: &str, args: Value) -> Result<Value> {
        let args_json = serde_json::to_string(&args)
            .map_err(BamlRtError::Json)?;
        
        let js_code = format!(
            r#"
            (function() {{
                try {{
                    const args = {};
                    if (typeof {} === 'function') {{
                        const promise = {}(args);
                        return __awaitAndStringify(promise);
                    }} else {{
                        const promise = __baml_invoke("{}", JSON.stringify(args));
                        return __awaitAndStringify(promise);
                    }}
                }} catch (error) {{
                    return JSON.stringify({{ error: error.toString() }});
                }}
            }})()
            "#,
            args_json, function_name, function_name, function_name
        );

        let mut bridge_guard = self.js_bridge.lock().await;
        let result = bridge_guard.evaluate(&js_code).await?;
        drop(bridge_guard);

        Ok(result)
    }
}

async fn load_agent_package(package_path: &std::path::Path) -> Result<LoadedAgent> {
    use baml_rt::quickjs_bridge::QuickJSBridge;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Extract package
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| BamlRtError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to get system time: {}", e)
        )))?;
    
    let extract_dir = std::env::temp_dir().join(format!("baml-agent-{}", timestamp.as_secs()));
    fs::create_dir_all(&extract_dir).map_err(BamlRtError::Io)?;

    let tar_gz = fs::File::open(package_path).map_err(BamlRtError::Io)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(&extract_dir).map_err(BamlRtError::Io)?;

    // Load manifest
    let manifest_path = extract_dir.join("manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path).map_err(BamlRtError::Io)?;
    let manifest_json: Value = serde_json::from_str(&manifest_content).map_err(BamlRtError::Json)?;

    let name = manifest_json.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| BamlRtError::InvalidArgument("manifest.json missing 'name' field".to_string()))?
        .to_string();

    let entry_point = manifest_json.get("entry_point")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "dist/index.js".to_string());

    // Load BAML schema
    let baml_src = extract_dir.join("baml_src");
    let baml_src_str = baml_src.to_str()
        .ok_or_else(|| BamlRtError::InvalidArgument(
            format!("BAML source path contains invalid UTF-8: {}", baml_src.display())
        ))?;
    let mut runtime_manager = baml_rt::baml::BamlRuntimeManager::new()?;
    runtime_manager.load_schema(baml_src_str)?;

    // Create QuickJS bridge
    let runtime_manager_arc = Arc::new(Mutex::new(runtime_manager));
    let mut js_bridge = QuickJSBridge::new(runtime_manager_arc.clone()).await?;
    js_bridge.register_baml_functions().await?;

    // Load agent JavaScript code
    let entry_point_path = extract_dir.join(&entry_point);
    if entry_point_path.exists() {
        let agent_code = fs::read_to_string(&entry_point_path).map_err(BamlRtError::Io)?;
        let _ = js_bridge.evaluate(&agent_code).await; // Initialize, ignore result
    }

    Ok(LoadedAgent {
        name,
        runtime_manager: runtime_manager_arc,
        js_bridge: Arc::new(Mutex::new(js_bridge)),
    })
}

