use async_ftp::{types::FileType, FtpStream};
use futures_util::StreamExt;
use std::cmp::min;
use std::io::Write;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;

use crate::address::ParsedAddress;
use crate::bar::WrappedBar;
use crate::hash::HashChecker;
use crate::io::get_output;
use crate::slicer::Slicer;

pub struct FTPHandler {
    pub output: Box<dyn Write>,
    pub transfered: u64,
}

struct FTPGetProperties {
    out: Box<dyn Write>,
    transfered: u64,
    total_size: u64,
    reader: tokio::io::BufReader<async_ftp::DataStream>,
}

impl FTPHandler {
    pub async fn get(
        input: &str,
        output: &str,
        bar: &mut WrappedBar,
        expected_sha256: &str,
        netrc: Option<netrc::Netrc>,
    ) -> bool {
        let _output = match output {
            "." => Slicer::target_with_extension(input),
            _ => output,
        };
        FTPHandler::_get(input, _output, bar, netrc).await;
        HashChecker::check(_output, expected_sha256, bar.silent)
    }

    async fn setup(
        input: &str,
        output: &str,
        bar: &mut WrappedBar,
        netrc: Option<netrc::Netrc>,
    ) -> Result<FTPGetProperties, Box<dyn std::error::Error>> {
        let (out, transfered) = get_output(output, bar.silent);

        let parsed_ftp = ParsedAddress::parse_address(input, netrc);
        let mut ftp_stream = FTPHandler::get_stream(transfered, &parsed_ftp)
            .await
            .unwrap();
        let total_size = ftp_stream.size(&parsed_ftp.file).await.unwrap().unwrap() as u64;

        bar.set_length(total_size);
        let reader = ftp_stream.get(&parsed_ftp.file).await.unwrap();
        Ok(FTPGetProperties {
            out,
            transfered,
            total_size,
            reader,
        })
    }

    async fn _get(input: &str, output: &str, bar: &mut WrappedBar, netrc: Option<netrc::Netrc>) {
        let mut properties = FTPHandler::setup(input, output, bar, netrc).await.unwrap();
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
                    properties.transfered + (byte_count as u64),
                    properties.total_size,
                );
                properties.transfered = new;
                bar.set_position(new);
            } else {
                break;
            }
        }

        bar.finish_download(&input, &output);
    }

    pub async fn put(input: &str, output: &str, mut bar: WrappedBar) -> bool {
        let file = tokio::fs::File::open(&input).await.unwrap();
        let total_size = file.metadata().await.unwrap().len();

        let parsed_ftp = ParsedAddress::parse_address(output, None);
        let transfered = 0;
        let mut ftp_stream = FTPHandler::get_stream(transfered, &parsed_ftp)
            .await
            .unwrap();
        let mut reader_stream = ReaderStream::new(file);

        bar.set_length(total_size);
        let mut uploaded = 0;

        let async_stream = async_stream::stream! {
            while let Some(chunk) = reader_stream.next().await {
                if let Ok(chunk) = &chunk {
                    let new = min(uploaded + (chunk.len() as u64), total_size);
                    uploaded = new;
                    bar.set_position(new);
                    if(uploaded >= total_size){
                        bar.finish_upload(&input, &output);
                    }
                }
                yield chunk;
            }
        };

        let stream_reader = tokio_util::io::StreamReader::new(async_stream);
        tokio::pin!(stream_reader);
        ftp_stream
            .put(&parsed_ftp.file, &mut stream_reader)
            .await
            .unwrap();

        true
    }

    async fn get_stream(
        transfered: u64,
        parsed_ftp: &ParsedAddress,
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
        ftp_stream.restart_from(transfered).await.unwrap();
        Ok(ftp_stream)
    }
}

#[tokio::test]
async fn get_ftp_works() {
    let out_file = "demo_README";
    let expected_hash = "1fda8bdf225ba614ce1e7db8830e4a2e9ee55907699521d500b1b7beff18523b";

    let result = FTPHandler::get(
        "ftp://ftp.fau.de:21/gnu/MailingListArchives/README",
        out_file,
        &mut WrappedBar::new_empty(),
        expected_hash,
        None,
    )
    .await;
    std::fs::remove_file(out_file).unwrap();

    assert!(result);
}

#[tokio::test]
async fn get_ftp_works_same_filename() {
    let out_file = ".";
    let expected_hash = "1fda8bdf225ba614ce1e7db8830e4a2e9ee55907699521d500b1b7beff18523b";

    let result = FTPHandler::get(
        "ftp://ftp.fau.de:21/gnu/MailingListArchives/README",
        out_file,
        &mut WrappedBar::new_empty(),
        expected_hash,
        None,
    )
    .await;
    std::fs::remove_file("README").unwrap();

    assert!(result);
}

#[tokio::test]
async fn get_ftp_resume_works() {
    let expected_size = 1370827;
    let out_file = "test/wpa_supplicant-2:2.9-8-x86_64.pkg.tar.zst";

    std::fs::copy(
        "test/incomplete_wpa_supplicant-2XX2.9-8-x86_64.pkg.tar.zst",
        out_file,
    )
    .unwrap();
    FTPHandler::get(
        "ftp://ftp.fau.de/archlinux/core/os/x86_64/wpa_supplicant-2:2.9-8-x86_64.pkg.tar.zst",
        out_file,
        &mut WrappedBar::new_empty(),
        "",
        None,
    )
    .await;

    let actual_size = std::fs::metadata(out_file).unwrap().len();
    assert_eq!(actual_size, expected_size);
    std::fs::remove_file(out_file).unwrap();
}
