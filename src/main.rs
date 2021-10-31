use clap::{clap_app, crate_version};

mod lib;

#[tokio::main]
async fn main() {
    let args = clap_app!(rcurl =>
        (version: crate_version!())
        (author: "Mihai Galos <mihaigalos at gmail dot com>")
        (about: "A simplified subset of curl written in Rust.")
        (@arg FILE: -O --output +takes_value "write documents to FILE")
        (@arg URL: +required +takes_value "url to download")
        )
        .get_matches_safe().unwrap_or_else(|e| e.exit());
    
    let url = args.value_of("URL").unwrap();
    let output_file = args.value_of("FILE").unwrap();

    lib::download_file(url, output_file).await.unwrap();
}
