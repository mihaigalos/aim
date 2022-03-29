use crate::error::ValidateError;
use warpy;

pub struct WarpyWrapper;

impl WarpyWrapper {
    pub async fn run(folder: String) -> Result<(), ValidateError> {
        let ip = [0, 0, 0, 0];
        let port = 8082;
        let footer = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        warpy::server::run(folder, ip, port, footer).await.unwrap();
        Ok(())
    }
}