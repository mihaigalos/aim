use autoclap::autoclap;
use clap::Command;
use clap::{Arg, ArgAction};
use std::os::unix::fs::PermissionsExt;
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
        )
        .arg(
            Arg::new("no-follow-redirects")
                .long("no-follow-redirects")
                .action(ArgAction::SetTrue)
                .help("Disable automatic following of HTTP redirects.")
                .required(false),
        )
        .arg(
            Arg::new("install")
                .long("install")
                .action(ArgAction::SetTrue)
                .help("Download, extract if needed, and install the binary to ~/.local/bin.")
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

    if args.get_flag("install") {
        let input_clone = input.clone();
        match install(&input_clone).await {
            Err(e) => {
                println!("ERROR: {e}");
                ::std::process::exit(1);
            }
            Ok(()) => ::std::process::exit(0),
        }
    }

    let output = args
        .get_one::<String>("OUTPUT")
        .map(|s| s.as_str())
        .unwrap_or("stdout");

    let silent = args.get_flag("silent");
    let interactive = args.get_flag("interactive");
    let no_follow_redirects = args.get_flag("no-follow-redirects");
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
            no_follow_redirects,
        },
    ))
}

#[cfg(not(tarpaulin_include))]
async fn install(input: &str) -> Result<(), Box<dyn ::std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&temp_dir)?;

    let filename = aim::slicer::Slicer::target_with_extension(input);
    let temp_file = temp_dir.join(filename);

    let options = aim::driver::Options {
        silent: false,
        interactive: false,
        expected_sha256: String::new(),
        no_follow_redirects: false,
    };
    aim::driver::Driver::dispatch(input, temp_file.to_str().unwrap(), &options).await?;

    let is_archive = filename.ends_with(".tar.gz")
        || filename.ends_with(".tar.xz")
        || filename.ends_with(".tar.bz2")
        || filename.ends_with(".zip")
        || filename.ends_with(".tar");

    let install_source = if is_archive {
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&temp_dir)?;
        melt::decompress(std::path::Path::new(temp_file.to_str().unwrap())).ok();
        std::env::set_current_dir(&original_dir)?;
        std::fs::remove_file(&temp_file).ok();
        find_binary_in_dir(&temp_dir)?
    } else {
        temp_file
    };

    let install_dir = std::path::PathBuf::from(untildify::untildify("~/.local/bin"));
    std::fs::create_dir_all(&install_dir)?;

    let binary_name = install_source.file_name().unwrap();
    let dest = install_dir.join(binary_name);
    std::fs::copy(&install_source, &dest)?;
    std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))?;

    println!("\u{2705} Installed to {}", dest.display());
    std::fs::remove_dir_all(&temp_dir).ok();
    Ok(())
}

#[cfg(not(tarpaulin_include))]
fn find_binary_in_dir(dir: &std::path::Path) -> Result<std::path::PathBuf, Box<dyn ::std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = entry.metadata()?;
        if meta.is_dir() {
            if let Ok(found) = find_binary_in_dir(&path) {
                return Ok(found);
            }
        } else if meta.is_file() && meta.permissions().mode() & 0o111 != 0 {
            return Ok(path);
        }
    }
    // Fallback: return first regular file
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            return Ok(entry.path());
        }
    }
    Err("No binary found in archive".into())
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

