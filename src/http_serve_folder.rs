use crate::error::ValidateError;
use warpy;

pub struct WarpyWrapper;

impl WarpyWrapper {
    pub async fn run(folder: String) -> Result<(), ValidateError> {
        let ip = [0, 0, 0, 0];
        let footer = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        warpy::server::run(folder, ip, footer, None, false)
            .await
            .unwrap();
        Ok(())
    }
}
