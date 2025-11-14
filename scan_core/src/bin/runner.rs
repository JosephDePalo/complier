use anyhow::Result;
use scan_core::{db::Db, scanner::Scanner};
use std::env;

use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL")?;
    let master_key = env::var("MASTER_KEY")?;
    let db = Db::new(db_url.as_str(), master_key.as_str()).await?;
    let scanner = Scanner::new(db)?;
    scanner.run().await?;

    Ok(())
}
