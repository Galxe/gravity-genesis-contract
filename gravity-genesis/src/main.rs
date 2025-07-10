use anyhow::Result;
use clap::Parser;
use gravity_genesis::execute::{self, GenesisConfig};
use tracing::{info, Level};
use tracing_subscriber;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.debug { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();

    info!("Starting Gravity Genesis Binary");

    // 读取Genesis配置文件
    info!("Reading Genesis configuration from: {}", args.config_file);
    let config_content = fs::read_to_string(&args.config_file)?;
    let config: GenesisConfig = serde_json::from_str(&config_content)?;
    info!("Genesis configuration loaded successfully");

    info!("Gravity Genesis Binary completed successfully");

    // 检查output dir是否设置，设置了检查对应的output dir是否存在 不存在则创建并且输出目录名
    if let Some(output_dir) = &args.output {
        if !fs::metadata(&output_dir).is_ok() {
            fs::create_dir_all(&output_dir).unwrap();
        }
        info!("Output directory: {}", output_dir);
    }

    execute::genesis_generate(&args.byte_code_dir, &args.output.unwrap_or("output".to_string()), config);

    Ok(())
} 