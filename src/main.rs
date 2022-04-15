extern crate clap;

use autoclap::autoclap;
use clap::Arg;
use clap::Command;
use std::env;

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() {
    let app: clap::Command = autoclap!();
    let args = app
        .arg(
            Arg::new("INPUT")
                .help("Input to aim from.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("OUTPUT")
            .help("Explicit output to aim to. \n\
            Can be ommitted during:\n\
            * Downloading: if filename supplied, writes to file, otherwise stdout (cannot resume).\n\
            * Uploading: directly uploads file to URL.\n\
            * if none present, writes to stdout.")
            .takes_value(true)
            .required(false),
        )
        .arg(
            Arg::new("SHA256")
                .help("Expected sha256 for verification. Will return a non-zero if mismatch.")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::new("silent")
                .long("silent")
                .short('s')
                .help("Silent or quiet mode. Don't show progress meter or error messages.")
                .required(false),
        )
        .try_get_matches()
        .unwrap_or_else(|e| e.exit());

    let input = args.value_of("INPUT").unwrap();
    let output = args.value_of("OUTPUT").unwrap_or("stdout");
    let expected_sha256 = args.value_of("SHA256").unwrap_or("");
    let silent = args.is_present("silent");

    match aim::driver::Driver::drive(input, output, silent, expected_sha256).await {
        Ok(_) => std::process::exit(0),
        _ => std::process::exit(255),
    }
}
