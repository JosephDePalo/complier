use std::io::Read;
use std::{fs, io};

use anyhow::Result;
use clap::Parser;
use cli::config::{Args, Config};
use scan_core::scanner::Scanner;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let contents = if let Some(path) = args.config {
        fs::read_to_string(path)?
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let config: Config = toml::from_str(&contents)?;
    let mut scanner = Scanner::new()?;
    for file_path in &config.settings.check_files {
        scanner.load_file(file_path)?;
    }
    scanner.exclude_checks(&config.settings.exclusion_ids)?;
    let db = scanner.scan_devices(config.devices).await?;
    let json = serde_json::to_string_pretty(&db)?;
    println!("{}", json);
    Ok(())
}
