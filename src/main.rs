use std::collections::HashMap;
use std::fs;

use rsaudit::config::Config;
use rsaudit::luainterp::LuaInterp;
use rsaudit::sshsession::SSHSession;

#[derive(Debug)]
struct Rules {
    passed: Vec<String>,
    failed: Vec<(String, String)>,
}

impl Rules {
    fn new() -> Self {
        Rules {
            passed: vec![],
            failed: vec![],
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let toml_str = fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&toml_str).unwrap();
    let mut database: HashMap<String, Rules> = HashMap::new();
    for device in config.devices {
        let sshses = SSHSession::new(
            device.address.as_str(),
            device.username.as_str(),
            device.password.as_str(),
        );
        database.insert(device.address.clone(), Rules::new());

        let lua = LuaInterp::new();
        lua.load_file("scripts/ssh_test.lua");
        let rule_id = String::from("1.2.3");
        let Some(f_is_bsd) = lua.get_func("is_bsd") else {
            eprintln!("Could not find function.");
            return Ok(());
        };
        let result: (bool, String) = f_is_bsd.call((sshses,)).unwrap();
        let Some(entry) = database.get_mut(device.address.as_str()) else {
            eprintln!("Could not find entry for '{}'", device.address.as_str());
            return Err(Box::<dyn std::error::Error>::from("Uh oh"));
        };
        if result.0 {
            entry.passed.push(rule_id);
        } else {
            entry.failed.push((rule_id, result.1.clone()));
        }

        println!("Result: {:?}", result);
    }

    println!("{:?}", database);

    Ok(())
}
