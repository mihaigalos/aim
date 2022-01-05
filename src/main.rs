use clap::{clap_app, crate_version};

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() {
    let args = clap_app!(aim =>
        (version: crate_version!())
        (author: "Mihai Galos <mihaigalos at gmail dot com>")
        (about: "🎯 A download/upload tool with resume.")
        (@arg silent: -s --silent "Silent or quiet mode. Don't show progress meter or error messages.")
        (@arg INPUT: +required +takes_value "Input to aim from.")
        (@arg OUTPUT: +takes_value "Explicit output to aim to. \n\
            Can be ommitted during:\n\
            * Downloading: if filename supplied, writes to file, otherwise stdout (cannot resume).\n\
            * Uploading: directly uploads file to URL.\n\
            * if none present, writes to stdout.")
        
        (@arg SHA256: +takes_value "Expected sha256 for verification. Will return a non-zero if mismatch")
    )
    .get_matches_safe()
    .unwrap_or_else(|e| e.exit());

    let input = args.value_of("INPUT").unwrap();
    let output = args.value_of("OUTPUT").unwrap_or("stdout");
    let silent = args.is_present("silent");
    let expected_sha256 = args.value_of("SHA256").unwrap_or("");

    let result  = aim::driver::Driver::drive(input, output, silent, expected_sha256).await;
    if result{
        std::process::exit(0);
    } else {
        std::process::exit(255);
    }
}
