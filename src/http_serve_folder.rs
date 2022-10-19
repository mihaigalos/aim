extern crate warpy;

use std::io::Result;

pub struct WarpyWrapper;

impl WarpyWrapper {
    pub async fn run(folder: String) -> Result<()> {
        let ip = [0, 0, 0, 0];
        let footer = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        warpy::server::run(folder, ip, footer, None, false).await?;

        Ok(())
    }
}
