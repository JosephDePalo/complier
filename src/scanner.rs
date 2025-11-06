use std::fs;
use std::sync::{Arc, Mutex};

use crate::luaregex::LuaRegex;
use anyhow::{Context, Result};
use mlua::{Function, Lua, Table, prelude::LuaError};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct CheckDefinition {
    id: String,
    name: String,
    description: String,
    severity: String,
    run: Function,
}

type CheckRegistry = Arc<Mutex<Vec<CheckDefinition>>>;

#[derive(Serialize, Deserialize)]
pub struct CheckResult {
    id: String,
    name: String,
    description: String,
    severity: String,
    compliant: bool,
    message: String,
}

impl CheckResult {
    pub fn from_check(
        check: &CheckDefinition,
        compliance: bool,
        msg: String,
    ) -> Self {
        Self {
            id: check.id.clone(),
            name: check.name.clone(),
            description: check.description.clone(),
            severity: check.severity.clone(),
            compliant: compliance,
            message: if msg != "" { msg } else { String::from("") },
        }
    }
}

pub type Database = Vec<CheckResult>;

pub struct Scanner {
    pub lua: Lua,
    pub registry: CheckRegistry,
}

impl Scanner {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();

        // Setup Registry and register_check function
        let registry: CheckRegistry = Arc::new(Mutex::new(Vec::new()));
        let registry_clone = Arc::clone(&registry);

        let register_check_fn = lua
            .create_function_mut(move |_, check_table: Table| {
                let check_def = CheckDefinition {
                    id: check_table.get("id")?,
                    name: check_table.get("name")?,
                    description: check_table.get("description")?,
                    severity: check_table.get("severity")?,
                    run: check_table.get("run")?,
                };
                let mut registry_guard =
                    registry_clone.lock().map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to acquire mutex: {}",
                            e
                        ))
                    })?;
                registry_guard.push(check_def);
                Ok(())
            })
            .context("Could not create 'register_check' function")?;

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
            .context("Could not create 'compile' function")?;

        let regex_module =
            lua.create_table().context("Could not create table")?;
        regex_module
            .set("compile", compile_fn)
            .context("Could not set 'compile' function in regex module")?;

        lua.globals()
            .set("regex", regex_module)
            .context("Could not set 'regex' global")?;

        lua.globals()
            .set("register_check", register_check_fn)
            .context("Could not set 'register_check' global")?;

        Ok(Scanner { lua, registry })
    }

    pub fn load_file(self: &Self, path: &str) -> mlua::Result<()> {
        let lua_code = fs::read_to_string(path)?;
        self.lua.load(&lua_code).exec()?;
        Ok(())
    }

    pub async fn run_checks(
        self: &Self,
        session: mlua::AnyUserData,
    ) -> Result<Database> {
        let checks = {
            self.registry
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire mutex: {}", e))?
                .clone()
        };
        let mut db: Database = vec![];
        for check in checks.iter() {
            let (status, msg): (bool, String) =
                check.run.call_async((session.clone(),)).await?;
            db.push(CheckResult::from_check(check, status, msg));
        }
        Ok(db)
    }

    pub fn exclude_checks(self: &Self, ids: &Vec<String>) -> Result<()> {
        let mut registry_guard = self
            .registry
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire mutex: {}", e))?;
        registry_guard.retain(|c| !ids.contains(&c.id));
        Ok(())
    }
}
