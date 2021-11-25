use clap::{crate_description, crate_version};

pub const CLIENT_ID: &str = concat!(crate_description!(), ':', crate_version!());
