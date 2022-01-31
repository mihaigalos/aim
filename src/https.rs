use futures_util::StreamExt;
use reqwest::Client;
use std::cmp::min;
use tokio_util::io::ReaderStream;

use crate::address::ParsedAddress;
use crate::bar::WrappedBar;
use crate::consts::*;
use crate::error::ValidateError;
use crate::hash::HashChecker;
use crate::io;
use crate::slicer::Slicer;

pub struct HTTPSHandler;
impl HTTPSHandler {
    pub async fn get(
        input: &str,
        output: &str,
        bar: &mut WrappedBar,
        expected_sha256: &str,
    ) -> Result<(), ValidateError> {
        let _output = match output {
            "." => Slicer::target_with_extension(input),
            _ => output,
        };
        HTTPSHandler::_get(input, _output, bar).await;
        HashChecker::check(_output, expected_sha256, bar.silent)
    }

    pub async fn put(input: &str, output: &str, mut bar: WrappedBar) -> Result<(), ValidateError> {
        let parsed_address = ParsedAddress::parse_address(output, bar.silent);
        let file = tokio::fs::File::open(&input)
            .await
            .expect("Cannot open input file for HTTPS read");
        let total_size = file
            .metadata()
            .await
            .expect("Cannot determine input file size for HTTPS read")
            .len();
        let input_ = input.to_string();
        let output_ = output.to_string();
        let mut reader_stream = ReaderStream::new(file);

        let mut uploaded = HTTPSHandler::get_already_uploaded(output, bar.silent).await;
        bar.set_length(total_size);

        let async_stream = async_stream::stream! {
            while let Some(chunk) = reader_stream.next().await {
                if let Ok(chunk) = &chunk {
                    let new = min(uploaded + (chunk.len() as u64), total_size);
                    uploaded = new;
                    bar.set_position(new);
                    if(uploaded >= total_size){
                        bar.finish_upload(&input_, &output_);
                    }
                }
                yield chunk;
            }
        };

        let _ = reqwest::Client::new()
            .put(output)
            .header("content-type", "application/octet-stream")
            .header("Range", "bytes=".to_owned() + &uploaded.to_string() + "-")
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static(CLIENT_ID),
            )
            .basic_auth(parsed_address.username, Some(parsed_address.password))
            .body(reqwest::Body::wrap_stream(async_stream))
            .send()
            .await
            .unwrap();
        Ok(())
    }

    async fn _get(input: &str, output: &str, bar: &mut WrappedBar) {
        let parsed_address = ParsedAddress::parse_address(input, bar.silent);
        let (mut out, mut downloaded) = io::get_output(output, bar.silent);

        let res = Client::new()
            .get(input)
            .header("Range", "bytes=".to_owned() + &downloaded.to_string() + "-")
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static(CLIENT_ID),
            )
            .basic_auth(parsed_address.username, Some(parsed_address.password))
            .send()
            .await
            .or(Err(format!("Failed to GET from {} to {}", &input, &output)))
            .unwrap();
        let total_size = downloaded + res.content_length().or(Some(0)).unwrap();

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

        bar.finish_download(&input, &output);
    }

    async fn get_already_uploaded(output: &str, silent: bool) -> u64 {
        let parsed_address = ParsedAddress::parse_address(output, silent);
        let res = Client::new()
            .get(output)
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static(CLIENT_ID),
            )
            .basic_auth(parsed_address.username, Some(parsed_address.password))
            .send()
            .await
            .or(Err(format!(
                "Failed to GET already uploaded size from {}",
                &output
            )))
            .unwrap();
        let uploaded = res.content_length().or(Some(0)).unwrap();
        uploaded
    }
}

#[tokio::test]
async fn get_https_works() {
    let expected_hash = "0e0f0d7139c8c7e3ff20cb243e94bc5993517d88e8be8d59129730607d5c631b";
    let out_file = "tokei-x86_64-unknown-linux-gnu.tar.gz";

    let result = HTTPSHandler::get("https://github.com/XAMPPRocky/tokei/releases/download/v12.0.4/tokei-x86_64-unknown-linux-gnu.tar.gz", out_file, &mut WrappedBar::new_empty(), expected_hash).await;

    assert!(result.is_ok());
    std::fs::remove_file(out_file).unwrap();
}

#[tokio::test]
async fn get_https_works_same_filename() {
    let expected_hash = "280a6edf58076e90d2002b44d38f93dcd708209c446dbcc38344ed6d21a8aaf7";
    let out_file = ".";

    let result = HTTPSHandler::get("https://github.com/casey/just/releases/download/0.10.2/just-0.10.2-x86_64-unknown-linux-musl.tar.gz", out_file, &mut WrappedBar::new_empty(), expected_hash).await;

    assert!(result.is_ok());
    std::fs::remove_file("just-0.10.2-x86_64-unknown-linux-musl.tar.gz").unwrap();
}

#[tokio::test]
async fn get_resume_works() {
    let expected_size = 561553;
    let out_file = "test/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz";
    std::fs::copy(
        "test/incomplete_dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz",
        out_file,
    )
    .unwrap();

    let _ = HTTPSHandler::get("https://github.com/Byron/dua-cli/releases/download/v2.10.2/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz", out_file, &mut WrappedBar::new_empty_verbose(), "").await;

    let actual_size = std::fs::metadata(out_file).unwrap().len();
    assert_eq!(actual_size, expected_size);
    std::fs::remove_file(out_file).unwrap();
}
