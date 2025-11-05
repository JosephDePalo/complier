use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

use mlua::{Function, Lua, Table, prelude::*};
use regex::Regex;
use rsaudit::config::Config;
use rsaudit::luaregex::LuaRegex;
use rsaudit::sshsession::SSHSession;

#[derive(Debug, Clone)]
pub struct CheckDefinition {
    id: String,
    name: String,
    description: String,
    severity: String,
    run: Function,
}

type CheckRegistry = Arc<Mutex<Vec<CheckDefinition>>>;

pub struct Scanner {
    lua: Lua,
    registry: CheckRegistry,
}

impl Scanner {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();

        // Setup Registry and register_check function
        let registry: CheckRegistry = Arc::new(Mutex::new(Vec::new()));
        let registry_clone = Arc::clone(&registry);

        let register_check_fn = lua.create_function_mut(move |_, check_table: Table| {
            let check_def = CheckDefinition {
                id: check_table.get("id")?,
                name: check_table.get("name")?,
                description: check_table.get("description")?,
                severity: check_table.get("severity")?,
                run: check_table.get("run")?,
            };
            let mut registry_guard = registry_clone.lock().unwrap();
            registry_guard.push(check_def);
            Ok(())
        })?;

        // Setup regex passthrough
        let compile_fn = lua
            .create_function(|_, pattern: String| {
                // Compile the regex in Rust
                match Regex::new(&pattern) {
                    // On success, wrap it in our UserData struct and return it
                    Ok(re) => Ok(LuaRegex(re)),
                    // On error, return the error message
                    Err(e) => Err(LuaError::runtime(e.to_string())),
                }
            })
            .unwrap();

        let regex_module = lua.create_table().unwrap();
        regex_module.set("compile", compile_fn).unwrap();

        lua.globals().set("regex", regex_module).unwrap();

        lua.globals().set("register_check", register_check_fn)?;

        Ok(Scanner { lua, registry })
    }

    pub fn load_file(self: &Self, path: &str) -> mlua::Result<()> {
        let lua_code = fs::read_to_string(path)?;
        self.lua.load(&lua_code).exec()?;
        Ok(())
    }

    pub fn list_checks(self: &Self) {
        let registry_guard = self.registry.lock().unwrap();
        for (i, check) in registry_guard.iter().enumerate() {
            println!("{}: {:?}", i, check);
        }
    }

    pub fn run_checks(self: &Self, session: mlua::AnyUserData) -> mlua::Result<()> {
        let registry_guard = self.registry.lock().unwrap();
        for check in registry_guard.iter() {
            println!("Running {}...", check.id);
            let (status, msg): (String, String) = check.run.call((session.clone(),))?;
            println!("Got back {} and {}", status, msg);
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let toml_str = fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&toml_str).unwrap();
    let scanner = Scanner::new().unwrap();
    scanner.load_file("scripts/my_checks.lua").unwrap();
    for device in config.devices {
        let session = SSHSession::new(
            device.address.as_str(),
            device.username.as_str(),
            device.password.as_str(),
        );
        let session_userdata = scanner.lua.create_userdata(session.clone())?;
        scanner.run_checks(session_userdata)?;
    }
    Ok(())
    // for device in config.devices {
    //     let sshses = SSHSession::new(
    //         device.address.as_str(),
    //         device.username.as_str(),
    //         device.password.as_str(),
    //     );
    //     database.insert(device.address.clone(), Rules::new());
    //
    //     lua.load_file("scripts/ssh_test.lua");
    //     let rule_id = String::from("1.2.3");
    //     let Some(f_is_bsd) = lua.get_func("is_bsd") else {
    //         eprintln!("Could not find function.");
    //         return Ok(());
    //     };
    //     let result: (bool, String) = f_is_bsd.call((sshses,)).unwrap();
    //     let Some(entry) = database.get_mut(device.address.as_str()) else {
    //         eprintln!("Could not find entry for '{}'", device.address.as_str());
    //         return Err(Box::<dyn std::error::Error>::from("Uh oh"));
    //     };
    //     if result.0 {
    //         entry.passed.push(rule_id);
    //     } else {
    //         entry.failed.push((rule_id, result.1.clone()));
    //     }
    //
    //     println!("Result: {:?}", result);
    // }
    //
    // println!("{:?}", database);
    //
    // Ok(())
}
