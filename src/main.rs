use clap::{clap_app, crate_version};

#[tokio::main]
async fn main() {
    let args = clap_app!(ship =>
    (version: crate_version!())
    (author: "Mihai Galos <mihaigalos at gmail dot com>")
    (about: "⛵ Download/upload tool written in Rust.")
    (@arg FILE: -O --output +takes_value "write documents to FILE. If not specified, writes to stdout (cannot resume).")
    (@arg URL: +required +takes_value "url to download")
    (@arg silent: -s --silent "Silent or quiet mode. Don't show progress meter or error messages.")
    )
    .get_matches_safe()
    .unwrap_or_else(|e| e.exit());
    let url = args.value_of("URL").unwrap();
    let output = args.value_of("FILE").unwrap_or("");
    let silent = args.is_present("silent");

    ship::driver::Driver::drive(url, output, silent).await;
}
