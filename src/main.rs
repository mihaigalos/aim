use std::cmp::min;
use std::fs::{File};
use std::io::{Write};
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;

use clap::{clap_app, crate_version};

fn get_file(path: &str) -> (std::fs::File, u64){
    let mut downloaded: u64 = 0;
    let file;
    if std::path::Path::new(path).exists() {
        println!("File exists. Resuming.");
        file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(path)
            .unwrap();

        let file_size = std::fs::metadata(path).unwrap().len();
        downloaded = file_size;
    } else {
        println!("Writing to new file.");
        file = File::create(path).or(Err(format!("Failed to create file '{}'", path))).unwrap();
    }
    (file, downloaded)
}

fn get_progress_bar(total_size: u64, url: &str) -> indicatif::ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("█  "));
    pb.set_message(&format!("Downloading {}", url));
    pb
}

pub async fn download_file(url: &str, path: &str) -> Result<(), String> {
    let (mut file, mut downloaded) = get_file(path);
    
    let res = Client::new() 
        .get(url)
        .header("Range","bytes=".to_owned()+&downloaded.to_string()+"-")
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let pb = get_progress_bar(total_size, url);

    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("Downloaded {} to {}", url, path));
    return Ok(());
}

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

    download_file(url, "big_buck_bunny_480p_20mb.mp4").await.unwrap();
}
