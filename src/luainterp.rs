use std::fs;

use crate::luaregex::LuaRegex;
use mlua::prelude::*;
use regex::Regex;

pub struct LuaInterp {
    lua: Lua,
}

impl LuaInterp {
    pub fn new() -> Self {
        let lua = Lua::new();

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

        Self { lua }
    }

    pub fn load_file(self: &Self, path: &str) {
        let lua_code = fs::read_to_string(path).unwrap();
        self.lua.load(&lua_code).exec().unwrap();
    }

    pub fn get_func(self: &Self, func_name: &str) -> Option<LuaFunction> {
        match self.lua.globals().get(func_name) {
            Ok(func) => Some(func),
            Err(_) => None,
        }
    }
}
