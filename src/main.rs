use std::io::Read;
use std::{fs, io};

use clap::Parser;
use rsaudit::config::{Args, Config};
use rsaudit::scanner::{Database, Scanner};
use rsaudit::sshsession::SSHSession;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let contents = if let Some(path) = args.config {
        fs::read_to_string(path)?
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let config: Config = toml::from_str(&contents).unwrap();
    let scanner = Scanner::new().unwrap();
    for file_path in &config.settings.check_files {
        scanner.load_file(file_path).unwrap();
    }
    scanner.exclude_checks(&config.settings.exclusion_ids);
    for device in config.devices {
        let session = SSHSession::new(
            device.address.as_str(),
            device.username.as_str(),
            device.password.as_str(),
        );
        let session_userdata = scanner.lua.create_userdata(session.clone())?;
        let mut db: HashMap<String, Database> = HashMap::new();
        db.insert(device.address, scanner.run_checks(session_userdata)?);
        let json = serde_json::to_string_pretty(&db)?;
        println!("{}", json);
    }
    Ok(())
}
