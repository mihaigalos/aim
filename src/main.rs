use clap::{clap_app, crate_version};

#[tokio::main]
async fn main() {
    let args = clap_app!(ship =>
    (version: crate_version!())
    (author: "Mihai Galos <mihaigalos at gmail dot com>")
    (about: "Download/upload tool written in Rust. ⛵")
    (@arg FILE: -O --output +takes_value "write documents to FILE. If not specified, writes to stdout (cannot resume).")
    (@arg URL: +required +takes_value "url to download")
    )
    .get_matches_safe()
    .unwrap_or_else(|e| e.exit());
    let url = args.value_of("URL").unwrap();
    let output_file = args.value_of("FILE").unwrap_or("");

    // ship::core::get(url, output_file).await.unwrap();
    ship::ftp::FTPHandler::get(url, output_file).await;
}
