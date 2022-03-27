use crate::error::ValidateError;
use warpy;

pub struct WarpyWrapper;

impl WarpyWrapper {
    pub async fn http_serve_folder(folder: String) -> Result<(), ValidateError> {
        let footer = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        warpy::server::run(folder, [0, 0, 0, 0], 8082, footer)
            .await
            .unwrap();
        Ok(())
    }
}
