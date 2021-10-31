use clap::{clap_app, crate_version};

mod lib;

#[tokio::main]
async fn main() {
    let args = clap_app!(rcurl =>
        (version: crate_version!())
        (author: "Mihai Galos <mihaigalos at gmail dot com>")
        (about: "A simplified subset of curl written in Rust.")
        (@arg URL: +required +takes_value "url to download")
        )
        .get_matches_safe().unwrap_or_else(|e| e.exit());
    
    let url = args.value_of("URL").unwrap();

    lib::download_file(url, "big_buck_bunny_480p_20mb.mp4").await.unwrap();
}
