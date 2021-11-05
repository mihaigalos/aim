use futures_util::StreamExt;
use reqwest::Client;
use std::cmp::min;

use crate::bar::get_progress_bar;
use crate::output::get_output;

pub async fn get(url: &str, path: &str) -> Result<(), String> {
    let (mut output, mut downloaded) = get_output(path);

    let res = Client::new()
        .get(url)
        .header("Range", "bytes=".to_owned() + &downloaded.to_string() + "-")
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = downloaded
        + res
            .content_length()
            .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let pb = get_progress_bar(total_size, url);

    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading.")))?;
        output
            .write_all(&chunk)
            .or(Err(format!("Error while writing to output.")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("⛵ Downloaded {} to {}", url, path));
    return Ok(());
}

#[tokio::test]
async fn get_works() {
    let expected_hash = "0e0f0d7139c8c7e3ff20cb243e94bc5993517d88e8be8d59129730607d5c631b";
    let out_file = "tokei-x86_64-unknown-linux-gnu.tar.gz";
    get("https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz", out_file).await.unwrap();

    let bytes = std::fs::read(out_file).unwrap();
    let computed_hash = sha256::digest_bytes(&bytes);
    assert_eq!(computed_hash, expected_hash);
    std::fs::remove_file(out_file).unwrap();
}

#[tokio::test]
async fn get_resume_works() {
    let expected_hash = "16c241b0446b2b8ae8851f3facacd7604fe4193b2c0a545ae07652300f63a1e8";
    let out_file = "test/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz";

    std::fs::copy(
        "test/incomplete_dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz",
        "test/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz",
    )
    .unwrap();
    get("https://github.com/Byron/dua-cli/releases/download/v2.10.2/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz", out_file).await.unwrap();

    let bytes = std::fs::read(out_file).unwrap();
    let computed_hash = sha256::digest_bytes(&bytes);
    assert_eq!(computed_hash, expected_hash);
    std::fs::remove_file(out_file).unwrap();
}
