use clap::{clap_app, crate_version};

#[tokio::main]
async fn main() {
    let args = clap_app!(ship =>
    (version: crate_version!())
    (author: "Mihai Galos <mihaigalos at gmail dot com>")
    (about: "⛵ A download/upload tool.")
    (@arg INPUT: +required +takes_value "Input to ship from.")
    (@arg OUTPUT: -o --output +takes_value "Explicit output to ship to. \n-o can be ommitted: if filename supplied, writes to file, otherwise stdout (cannot resume).")
    (@arg OUTPUT_ALT: -O +takes_value "Alternative to -o.")
    (@arg silent: -s --silent "Silent or quiet mode. Don't show progress meter or error messages.")
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
