use mlua::{UserData, UserDataMethods, prelude::*};
use regex::Regex;

pub struct LuaRegex(pub Regex);

impl UserData for LuaRegex {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Expose regex:is_match(text)
        methods.add_method("is_match", |_, this, text: String| {
            Ok(this.0.is_match(&text))
        });

        // Expose regex:find(text) -> "the_match" or nil
        methods.add_method("find", |_, this, text: String| {
            // Find the first match and return it as a string, or nil
            let result = this.0.find(&text).map(|m| m.as_str().to_string());
            Ok(result)
        });

        // Expose regex:captures(text) -> { "full", "cap1", "cap2" } or nil
        methods.add_method("captures", |lua, this, text: String| {
            if let Some(caps) = this.0.captures(&text) {
                // Create a new Lua table
                let tbl = lua.create_table()?;
                // Iterate over captures and add them to the table
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
                // No captures found, return nil
                Ok(None)
            }
        });
    }
}
