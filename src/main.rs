use autoclap::autoclap;
use clap::Command;
use clap::{Arg, ArgAction};
use std::{env, io};

use aim::driver::Options;

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() {
    let (input, output, options) = parse_args().await.expect("Cannot parse args");
    match aim::driver::Driver::dispatch(&input, &output, &options).await {
        Ok(_) => std::process::exit(0),
        _ => std::process::exit(255),
    }
}

#[cfg(not(tarpaulin_include))]
async fn parse_args() -> io::Result<(String, String, Options)> {
    let app: clap::Command = autoclap!()
        .arg(
            Arg::new("INPUT")
                .help(
                    "Input to aim from.\n\
                If no output provided and input is a folder, it will be served via http.",
                )
                .required(false),
        )
        .arg(
            Arg::new("OUTPUT")
                .help(
                    "Explicit output to aim to. \n\
            * If no output argument is present, writes to stdout.\n\
            * Downloading: if file supplied, writes to it.\n\
              \x20\x20* if output is '.': downloads to the same basename as the source.\n\
              \x20\x20* if output is '+': downloads to the same basename as the source \n\
              \x20\x20\x20\x20and attempts to decompress the archive based on the file's extension.\n\
            * Uploading: directly uploads file to the URL.",
                )
                .required(false),
        )
        .arg(
            Arg::new("SHA256")
                .help("Expected sha256 for verification. Will return a non-zero if mismatch.")
                .required(false),
        )
       .arg(
           Arg::new("version")
               .long("version")
               .short('v')
               .action(ArgAction::SetTrue)
               .help("Prints current version.")
               .required(false),
       )
        .arg(
            Arg::new("silent")
                .long("silent")
                .short('s')
                .action(ArgAction::SetTrue)
                .help("Silent or quiet mode. Don't show progress meter or error messages.")
                .required(false),
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .short('i')
                .action(ArgAction::SetTrue)
                .help("Navigate folder structure in remote, interactively.\n\
            Use Tab, / to enter a folder, .. to exit, Enter to accept selection.")
                .required(false),
        )
        .arg(
            Arg::new("update")
                .long("update")
                .short('u')
                .action(ArgAction::SetTrue)
                .help("Update the executable in-place.")
                .required(false),
        );
    let args = app.clone().try_get_matches().unwrap_or_else(|e| e.exit());

    if args.get_flag("update") {
        tokio::task::spawn_blocking(move || match update() {
            Err(e) => {
                println!("ERROR: {e}");
                ::std::process::exit(1);
            }
            Ok(()) => ::std::process::exit(0),
        })
        .await
        .unwrap();
    }

    if args.get_flag("version") {
        println!("{}", app.get_about().unwrap());
    }
    let input = args
        .get_one::<String>("INPUT")
        .unwrap_or_else(|| ::std::process::exit(0));

    let output = args
        .get_one::<String>("OUTPUT")
        .map(|s| s.as_str())
        .unwrap_or("stdout");

    let silent = args.get_flag("silent");
    let interactive = args.get_flag("interactive");
    let expected_sha256 = args
        .get_one::<String>("SHA256")
        .map(|s| s.as_str())
        .unwrap_or("");

    Ok((
        input.to_string(),
        output.to_string(),
        Options {
            silent,
            interactive,
            expected_sha256: expected_sha256.to_string(),
        },
    ))
}

#[cfg(not(tarpaulin_include))]
fn update() -> Result<(), Box<dyn ::std::error::Error>> {
    let _status = self_update::backends::github::Update::configure()
        .repo_owner("mihaigalos")
        .repo_name(env!("CARGO_PKG_NAME"))
        .bin_name(env!("CARGO_PKG_NAME"))
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    println!("✅ Done.");
    Ok(())
}

