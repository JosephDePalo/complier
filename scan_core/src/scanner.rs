pub mod checks;
pub mod lua;
pub mod ssh;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use mlua::{AnyUserData, Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tokio::task::JoinHandle;

use crate::{
    db::models::{CheckStatus, ScanStatus},
    scanner::ssh::SSHSession,
};
use crate::{
    db::{Db, models::Device},
    scanner::lua::init_lua,
};

type ScanResult = HashMap<String, Vec<CheckResult>>;

#[derive(Debug, Deserialize)]
struct CheckResult {
    status: CheckStatus,
    details: Option<String>,
}

pub struct Scanner {
    lua: Lua,
    pub db: Db,
}

impl Scanner {
    pub fn new(db: Db) -> Result<Self> {
        let lua = init_lua()?;
        Ok(Self { lua, db })
    }

    pub async fn run(self: &Self) -> Result<()> {
        let rules = self.db.get_all_rules().await?;
        let devices = self.db.get_all_devices().await?;

        let lua = Arc::new(self.lua.clone());
        let db = Arc::new(self.db.clone());
        let rules = Arc::new(rules);

        let mut handles = Vec::new();
        for device in devices {
            let lua = lua.clone();
            let db = db.clone();
            let rules = rules.clone();
            let device = device.clone();

            let handle: JoinHandle<Result<()>> =
                tokio::task::spawn(async move {
                    let session = SSHSession::new(
                        device.address.as_str(),
                        device.username.as_str(),
                        device.password.as_str(),
                    )
                    .await?;

                    lua.globals().set("conn", session)?;

                    for rule in rules.iter() {
                        let scan =
                            db.add_scan(device.id, ScanStatus::Running).await?;
                        let table: Value =
                            lua.load(&rule.script_body).eval_async().await?;
                        let result: CheckResult = lua.from_value(table)?;
                        println!("Result: {:?}", &result);

                        db.add_scan_result(
                            scan.id,
                            rule.id.clone(),
                            result.status,
                            result.details,
                        )
                        .await?;
                    }
                    Ok(())
                });
            handles.push(handle);
        }

        for handle in handles {
            handle.await??;
        }

        Ok(())
    }
    // pub async fn scan_devices(
    //     self: &mut Self,
    //     devices: Vec<Device>,
    // ) -> Result<ScanResult> {
    //     let mut handles = Vec::new();
    //     let runner = Arc::new(
    //         self.runner
    //             .lock()
    //             .map_err(|e| anyhow::anyhow!("Failed to acquire mutex: {}", e))?
    //             .clone(),
    //     );
    //
    //     for device in devices {
    //         let runner = runner.clone();
    //         let handle: JoinHandle<Result<(String, Vec<CheckResult>)>> =
    //             tokio::task::spawn(async move {
    //                 let session = SSHSession::new(
    //                     device.address.as_str(),
    //                     device.username.as_str(),
    //                     device.password.as_str(),
    //                 )
    //                 .await?;
    //                 let session_userdata =
    //                     runner.lua.create_userdata(session)?;
    //                 Ok((
    //                     device.address,
    //                     runner.run_checks(session_userdata).await?,
    //                 ))
    //             });
    //         handles.push(handle);
    //     }
    //
    //     let mut db: ScanResult = HashMap::new();
    //     for handle in handles {
    //         if let Ok(handle) = handle.await {
    //             match handle {
    //                 Ok((address, result)) => {
    //                     db.insert(address, result);
    //                 }
    //                 Err(e) => eprintln!("An error occurred: {}", e),
    //             }
    //         }
    //     }
    //
    //     Ok(db)
    // }
}
