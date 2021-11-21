use futures_util::StreamExt;
use reqwest::Client;
use std::cmp::min;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::bar::WrappedBar;
use crate::output::get_output;

pub struct HTTPSHandler;
impl HTTPSHandler {
    pub async fn get(input: &str, output: &str, bar: &WrappedBar) {
        let (mut out, mut downloaded) = get_output(output, bar.silent);

        let res = Client::new()
            .get(input)
            .header("Range", "bytes=".to_owned() + &downloaded.to_string() + "-")
            .send()
            .await
            .or(Err(format!("Failed to GET from {} to {}", &input, &output)))
            .unwrap();
        let total_size = downloaded
            + res
                .content_length()
                .ok_or(format!("Failed to get content length from '{}'", &input))
                .unwrap();

        bar.set_length(total_size);

        let mut stream = res.bytes_stream();
        while let Some(item) = stream.next().await {
            let chunk = item.or(Err(format!("Error while downloading."))).unwrap();
            out.write_all(&chunk)
                .or(Err(format!("Error while writing to output.")))
                .unwrap();
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            bar.set_position(new);
        }

        bar.finish_with_message(format!("⛵ Downloaded {} to {}", input, output));
    }
    pub async fn put(input: &str, output: &str, _: &WrappedBar) {
        println!("{} -> {}", input, output);
        let mut file = File::open(input).await.unwrap();
        let mut vec = Vec::new();
        let _ = file.read_to_end(&mut vec);

        let stream = FramedRead::new(file, BytesCodec::new());
        let body = reqwest::Body::wrap_stream(stream);

        let _ = Client::new()
            .put(output)
            .header("content-type", "application/octet-stream")
            .body(body)
            .send()
            .await
            .unwrap();

        //bar.set_length(total_size);
        // bar.finish_with_message(format!("⛵ Uploaded {} to {}", input, output));
    }

    //pub async fn put(input: &str, output: &str, _: &WrappedBar) {
    //    println!("{} -> {}", input, output);
    //    let mut file = File::open(input).await.unwrap();
    //    let mut vec = Vec::new();
    //    let _ = file.read_to_end(&mut vec);
    //    let res = Client::new()
    //        .put(output)
    //        .header("content-type", "application/octet-stream")
    //        .body(vec)
    //        .send()
    //        .await
    //        .unwrap();
    //
    //    //bar.set_length(total_size);
    //    // bar.finish_with_message(format!("⛵ Uploaded {} to {}", input, output));
    //}
}
#[tokio::test]
async fn get_works() {
    let expected_hash = "0e0f0d7139c8c7e3ff20cb243e94bc5993517d88e8be8d59129730607d5c631b";
    let out_file = "tokei-x86_64-unknown-linux-gnu.tar.gz";
    HTTPSHandler::get("https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz", out_file, &WrappedBar::new_empty()).await;

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
    HTTPSHandler::get("https://github.com/Byron/dua-cli/releases/download/v2.10.2/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz", out_file, &WrappedBar::new_empty()).await;

    let bytes = std::fs::read(out_file).unwrap();
    let computed_hash = sha256::digest_bytes(&bytes);
    assert_eq!(computed_hash, expected_hash);
    std::fs::remove_file(out_file).unwrap();
}
