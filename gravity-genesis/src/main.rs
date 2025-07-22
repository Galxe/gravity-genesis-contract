use anyhow::Result;
use clap::Parser;
use gravity_genesis::execute::{self, GenesisConfig};
use tracing::{info, Level};
use std::fs;
use serde_json;

// Custom guard to ensure proper log flushing
struct LogGuard {
    _guard: Option<tracing_appender::non_blocking::WorkerGuard>,
    has_file_logging: bool,
}

impl LogGuard {
    fn new(guard: Option<tracing_appender::non_blocking::WorkerGuard>) -> Self {
        let has_file_logging = guard.is_some();
        Self {
            _guard: guard,
            has_file_logging,
        }
    }

    fn flush_and_wait(&self) {
        if self.has_file_logging {
            tracing::info!("Ensuring all logs are written to file...");
            // The drop of _guard will signal the background thread to finish
            // We give it some time to complete
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}

impl Drop for LogGuard {
    fn drop(&mut self) {
        if self.has_file_logging {
            // Ensure logs are flushed when guard is dropped
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Byte code directory
    #[arg(short, long)]
    byte_code_dir: String,

    /// Genesis configuration file
    #[arg(short, long, default_value = "generate/genesis_config.json")]
    config_file: String,

    /// Save results to file
    #[arg(short, long)]
    output: Option<String>,

    /// Log file path (optional)
    #[arg(short, long)]
    log_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.debug { Level::DEBUG } else { Level::INFO };
    
    // Set up logging and create log guard for proper cleanup
    let log_guard = if let Some(log_file_path) = &args.log_file {
        // Create log file directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(log_file_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Set up logging to file
        let file_appender = tracing_appender::rolling::never("", log_file_path);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();
            
        info!("Logging to file: {}", log_file_path);
        LogGuard::new(Some(guard))
    } else {
        // Console-only logging
        tracing_subscriber::fmt()
            .with_max_level(level)
            .init();
        LogGuard::new(None)
    };

    // Set up panic hook to ensure logs are flushed before panic
    let has_file_logging = log_guard.has_file_logging;
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        if has_file_logging {
            eprintln!("PANIC occurred! Ensuring all logs are written...");
            // Log the panic information
            tracing::error!("PANIC: {}", panic_info);
            tracing::error!("Flushing logs before panic exit...");
            
            // Give time for the background thread to write logs
            std::thread::sleep(std::time::Duration::from_millis(1200));
            eprintln!("Log flush attempt completed");
        }
        original_hook(panic_info);
    }));

    info!("Starting Gravity Genesis Binary");

    // Run the main logic
    let result = run_main_logic(&args).await;
    
    // Ensure logs are flushed before exiting
    info!("Main execution completed");
    log_guard.flush_and_wait();
    
    result
}

async fn run_main_logic(args: &Args) -> Result<()> {
    info!("Reading Genesis configuration from: {}", args.config_file);
    let config_content = fs::read_to_string(&args.config_file)?;
    let config: GenesisConfig = serde_json::from_str(&config_content)?;
    info!("Genesis configuration loaded successfully");

    if let Some(output_dir) = &args.output {
        if !fs::metadata(&output_dir).is_ok() {
            fs::create_dir_all(&output_dir).unwrap();
        }
        info!("Output directory: {}", output_dir);
    }

    execute::genesis_generate(&args.byte_code_dir, &args.output.as_ref().unwrap_or(&"output".to_string()), config);

    info!("Gravity Genesis Binary completed successfully");
    Ok(())
} 