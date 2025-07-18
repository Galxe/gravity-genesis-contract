use anyhow::Result;
use clap::Parser;
use gravity_genesis::execute::{self, GenesisConfig};
use tracing::{info, Level};
use std::fs;
use serde_json;

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
    
    // Configure logging based on whether log file is specified
    if let Some(log_file_path) = &args.log_file {
        // Create log file directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(log_file_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Set up logging to both file and console
        let file_appender = tracing_appender::rolling::never("", log_file_path);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();
            
        info!("Logging to file: {}", log_file_path);
    } else {
        // Console-only logging
        tracing_subscriber::fmt()
            .with_max_level(level)
            .init();
    }

    info!("Starting Gravity Genesis Binary");

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

    execute::genesis_generate(&args.byte_code_dir, &args.output.unwrap_or("output".to_string()), config);

    info!("Gravity Genesis Binary completed successfully");
    Ok(())
} 