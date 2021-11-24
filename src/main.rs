use clap::{clap_app, crate_version};

#[tokio::main]
async fn main() {
    let args = clap_app!(ship =>
    (version: crate_version!())
    (author: "Mihai Galos <mihaigalos at gmail dot com>")
    (about: "⛵ A download/upload tool with resume.")
    (@arg silent: -s --silent "Silent or quiet mode. Don't show progress meter or error messages.")
    (@arg INPUT: +required +takes_value "Input to ship from.")
    (@arg OUTPUT: +takes_value "Explicit output to ship to. \nCan be ommitted during:\n  Downloading: if filename supplied, writes to file, otherwise stdout (cannot resume).\n  Uploading: directly uploads file to URL.")
    )
    .get_matches_safe()
    .unwrap_or_else(|e| e.exit());
    let input = args.value_of("INPUT").unwrap();
    let mut output = args.value_of("OUTPUT").unwrap_or("");
    if output == "" {
        output = args.value_of("OUTPUT_ALT").unwrap_or("");
    }

    let silent = args.is_present("silent");

    ship::driver::Driver::drive(input, output, silent).await;
}
