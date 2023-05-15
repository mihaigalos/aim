extern crate warpy;

use dotenvy::dotenv;
use std::env;
use std::io::Result;

pub struct WarpyWrapper;

const DEFAULT_AIM_HOSTING_PORT: Option<u16> = None;

impl WarpyWrapper {
    pub async fn run(folder: String) -> Result<()> {
        dotenv().ok();
        let ip = [0, 0, 0, 0];
        let port: Option<u16> = match &env::var("AIM_HOSTING_PORT") {
            Ok(e) => Some(e.parse::<u16>().unwrap()),
            Err(_) => DEFAULT_AIM_HOSTING_PORT
        };
        let footer = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        warpy::server::run(folder, ip, footer, port, false).await?;

        Ok(())
    }
}
