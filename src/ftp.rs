use async_ftp::{types::FileType, FtpStream};
use failure::format_err;
use std::cmp::min;
use std::io::Write;
use tokio::io::AsyncReadExt;
use url::Url;

use crate::bar::get_progress_bar;
use crate::output::get_output;

pub struct FTPHandler {
    pub output: Box<dyn Write>,
    pub downloaded: u64,
}

fn parse_ftp_address(address: &str) -> (String, String, String, Vec<String>, String) {
    let url = Url::parse(address).unwrap();
    let ftp_server = format!(
        "{}:{}",
        url.host_str()
            .ok_or_else(|| format_err!("failed to parse hostname from url: {}", url))
            .unwrap(),
        url.port_or_known_default()
            .ok_or_else(|| format_err!("failed to parse port from url: {}", url))
            .unwrap(),
    );
    let username = if url.username().is_empty() {
        "anonymous".to_string()
    } else {
        url.username().to_string()
    };
    let password = url.password().unwrap_or("anonymous").to_string();

    let mut path_segments: Vec<String> = url
        .path_segments()
        .ok_or_else(|| format_err!("failed to get url path segments: {}", url))
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let file = path_segments
        .pop()
        .ok_or_else(|| format_err!("got empty path segments from url: {}", url))
        .unwrap();

    (
        ftp_server,
        username,
        password,
        path_segments,
        file.to_string(),
    )
}

impl FTPHandler {
    pub async fn get(url: &str, path: &str) {
        let (mut output, mut downloaded) = get_output(path);

        let (ftp_server, ref username, ref password, path_segments, ref file) =
            parse_ftp_address(url);

        let mut ftp_stream = FtpStream::connect(ftp_server).await.unwrap();
        let _ = ftp_stream.login(username, password).await.unwrap();

        for path in &path_segments {
            ftp_stream.cwd(&path).await.unwrap();
        }

        ftp_stream.transfer_type(FileType::Binary).await.unwrap();
        let total_size = ftp_stream.size(file).await.unwrap().unwrap() as u64;
        ftp_stream.restart_from(downloaded).await.unwrap();

        let pb = get_progress_bar(total_size, url);

        let mut reader = ftp_stream.get(file).await.unwrap();
        loop {
            let mut buffer = vec![0; 1024usize];
            let byte_count = reader.read(&mut buffer[..]).await.unwrap();
            buffer.truncate(byte_count);
            if !buffer.is_empty() {
                output
                    .write_all(&buffer)
                    .or(Err(format!("Error while writing to output.")))
                    .unwrap();
                let new = min(downloaded + (byte_count as u64), total_size);
                downloaded = new;
                pb.set_position(new);
            } else {
                break;
            }
        }

        pb.finish_with_message(&format!("⛵ Downloaded {} to {}", url, path));
    }
}

#[tokio::test]
async fn get_ftp_works() {
    let out_file = "demo_README";
    let expected_hash = "1fda8bdf225ba614ce1e7db8830e4a2e9ee55907699521d500b1b7beff18523b";

    FTPHandler::get(
        "ftp://ftp.fau.de:21/gnu/MailingListArchives/README",
        out_file,
    )
    .await;
    let bytes = std::fs::read(out_file).unwrap();
    let computed_hash = sha256::digest_bytes(&bytes);
    assert_eq!(computed_hash, expected_hash);
    std::fs::remove_file(out_file).unwrap();
}

#[tokio::test]
async fn get_ftp_resume_works() {
    let expected_hash = "1f48212d6c9d3fc38d2b9c81805078108ed771dc811b4a8f8ec8ac2a56646994";
    let out_file = "test/wpa_supplicant-2:2.9-8-x86_64.pkg.tar.zst";

    std::fs::copy(
        "test/incomplete_wpa_supplicant-2:2.9-8-x86_64.pkg.tar.zst",
        out_file,
    )
    .unwrap();
    FTPHandler::get(
        "ftp://ftp.fau.de/archlinux/core/os/x86_64/wpa_supplicant-2:2.9-8-x86_64.pkg.tar.zst",
        out_file,
    )
    .await;

    let bytes = std::fs::read(out_file).unwrap();
    let computed_hash = sha256::digest_bytes(&bytes);
    assert_eq!(computed_hash, expected_hash);
    std::fs::remove_file(out_file).unwrap();
}
