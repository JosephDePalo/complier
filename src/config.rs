use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub devices: Vec<Device>,
    pub settings: Settings,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    pub address: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub exclusion_ids: Vec<String>,
    pub check_files: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(name = "rsaudit", version, about = "Example CLI tool")]
pub struct Args {
    /// Path to config file (optional)
    #[arg(short, long)]
    pub config: Option<String>,
}
