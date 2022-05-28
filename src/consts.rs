use clap::crate_version;

pub const CLIENT_ID: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/tag/",
    crate_version!()
);
pub const BUFFER_SIZE: usize = 26_214_400;

#[cfg(debug_assertions)]
pub const AUTO_ALLOW_HTTP: bool = true;

#[cfg(not(debug_assertions))]
pub const AUTO_ALLOW_HTTP: bool = false;
