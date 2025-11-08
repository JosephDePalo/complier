use anyhow::Result;
use mlua::{UserData, UserDataMethods, prelude::*};
use regex::Regex;

pub fn init_lua() -> Result<Lua> {
    let lua = Lua::new();

    let compile_fn = lua
        .create_function(|_, pattern: String| match Regex::new(&pattern) {
            Ok(re) => Ok(LuaRegex(re)),
            Err(e) => Err(LuaError::runtime(e.to_string())),
        })
        .context("Could not create 'compile' function")?;

    let regex_module = lua.create_table().context("Could not create table")?;
    regex_module
        .set("compile", compile_fn)
        .context("Could not set 'compile' function in regex module")?;

    lua.globals()
        .set("regex", regex_module)
        .context("Could not set 'regex' global")?;

    Ok(lua)
}

pub struct LuaRegex(pub Regex);

impl UserData for LuaRegex {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Expose regex:is_match(text)
        methods.add_method("is_match", |_, this, text: String| {
            Ok(this.0.is_match(&text))
        });

        // Expose regex:find(text) -> "the_match" or nil
        methods.add_method("find", |_, this, text: String| {
            let result = this.0.find(&text).map(|m| m.as_str().to_string());
            Ok(result)
        });

        // Expose regex:captures(text) -> { "full", "cap1", "cap2" } or nil
        methods.add_method("captures", |lua, this, text: String| {
            if let Some(caps) = this.0.captures(&text) {
                let tbl = lua.create_table()?;
                for (i, mat) in caps.iter().enumerate() {
                    tbl.set(
                        i,
                        mat.map_or(LuaValue::Nil, |m| {
                            LuaValue::String(
                                lua.create_string(m.as_str()).unwrap(),
                            )
                        }),
                    )?;
                }
                Ok(Some(tbl))
            } else {
                Ok(None)
            }
        });
    }
}
