use async_ftp::{types::FileType, FtpStream};
use failure::format_err;
use std::cmp::min;
use std::io::Write;
use tokio::io::AsyncReadExt;
use url::Url;

use crate::bar::WrappedBar;
use crate::output::get_output;

pub struct FTPHandler {
    pub output: Box<dyn Write>,
    pub downloaded: u64,
}

#[derive(Debug)]
struct FTPParsedAddress {
    server: String,
    username: String,
    password: String,
    path_segments: Vec<String>,
    file: String,
}

impl PartialEq for FTPParsedAddress {
    fn eq(&self, other: &Self) -> bool {
        let result = self.server == other.server
            && self.username == other.username
            && self.password == other.password
            && self.file == other.file;

        let mut paths_equal = true;
        for it in self.path_segments.iter().zip(self.path_segments.iter()) {
            let (left, right) = it;
            paths_equal = paths_equal && (left == right);
        }

        result && paths_equal
    }
}

fn parse_ftp_address(address: &str) -> FTPParsedAddress {
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

    FTPParsedAddress {
        server: ftp_server,
        username: username,
        password: password,
        path_segments: path_segments,
        file: file.to_string(),
    }
}

async fn get_stream(
    downloaded: u64,
    parsed_ftp: &FTPParsedAddress,
) -> Result<async_ftp::FtpStream, async_ftp::FtpError> {
    let mut ftp_stream = FtpStream::connect((*parsed_ftp).server.clone())
        .await
        .unwrap();
    let _ = ftp_stream
        .login(&parsed_ftp.username, &parsed_ftp.password)
        .await
        .unwrap();

    for path in &parsed_ftp.path_segments {
        ftp_stream.cwd(&path).await.unwrap();
    }

    ftp_stream.transfer_type(FileType::Binary).await.unwrap();
    ftp_stream.restart_from(downloaded).await.unwrap();
    Ok(ftp_stream)
}
struct FTPProperties {
    out: Box<dyn Write>,
    downloaded: u64,
    total_size: u64,
    reader: tokio::io::BufReader<async_ftp::DataStream>,
}

impl FTPHandler {
    pub async fn get(input: &str, output: &str, bar: &mut WrappedBar) {
        let mut properties = FTPHandler::setup(input, output, bar).await.unwrap();
        loop {
            let mut buffer = vec![0; 26214400usize];
            let byte_count = properties.reader.read(&mut buffer[..]).await.unwrap();

            buffer.truncate(byte_count);
            if !buffer.is_empty() {
                properties
                    .out
                    .write_all(&buffer)
                    .or(Err(format!("Error while writing to output.")))
                    .unwrap();
                let new = min(
                    properties.downloaded + (byte_count as u64),
                    properties.total_size,
                );
                properties.downloaded = new;
                bar.set_position(new);
            } else {
                break;
            }
        }

        bar.finish_with_message(format!("🎯 Downloaded {} to {}", input, output));
    }
    pub async fn put(_: &str, _: &str, _: &WrappedBar) {}

    async fn setup(
        input: &str,
        output: &str,
        bar: &mut WrappedBar,
    ) -> Result<FTPProperties, Box<dyn std::error::Error>> {
        let (out, downloaded) = get_output(output, bar.silent);

        let parsed_ftp = parse_ftp_address(input);
        let mut ftp_stream = get_stream(downloaded, &parsed_ftp).await.unwrap();
        let total_size = ftp_stream.size(&parsed_ftp.file).await.unwrap().unwrap() as u64;

        bar.set_length(total_size);
        let reader = ftp_stream.get(&parsed_ftp.file).await.unwrap();
        Ok(FTPProperties {
            out,
            downloaded,
            total_size,
            reader,
        })
    }
}

#[tokio::test]
async fn ftpparseaddress_operator_equals_works_when_typical() {
    let left = FTPParsedAddress {
        server: "do.main".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["my".to_string(), "path".to_string()],
        file: "pass".to_string(),
    };
    let right = FTPParsedAddress {
        server: "do.main".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["my".to_string(), "path".to_string()],
        file: "pass".to_string(),
    };

    assert!(left == right);
}

#[tokio::test]
async fn ftpparseaddress_operator_equals_fails_when_not_equal() {
    let left = FTPParsedAddress {
        server: "do.main".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["my".to_string(), "path".to_string()],
        file: "pass".to_string(),
    };
    let right = FTPParsedAddress {
        server: "do".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["my".to_string(), "path".to_string()],
        file: "pass".to_string(),
    };

    assert!(left != right);
}

#[tokio::test]
async fn parse_ftp_works() {
    let expected = FTPParsedAddress {
        server: "do.main:21".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["index".to_string()],
        file: "file".to_string(),
    };

    let actual = parse_ftp_address("ftp://user:pass@do.main:21/index/file");

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn get_ftp_works() {
    let out_file = "demo_README";
    let expected_hash = "1fda8bdf225ba614ce1e7db8830e4a2e9ee55907699521d500b1b7beff18523b";

    FTPHandler::get(
        "ftp://ftp.fau.de:21/gnu/MailingListArchives/README",
        out_file,
        &mut WrappedBar::new_empty(),
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
        &mut WrappedBar::new_empty(),
    )
    .await;

    let bytes = std::fs::read(out_file).unwrap();
    let computed_hash = sha256::digest_bytes(&bytes);
    assert_eq!(computed_hash, expected_hash);
    std::fs::remove_file(out_file).unwrap();
}
