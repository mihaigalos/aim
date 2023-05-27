extern crate ssh2;

use async_ssh2_lite::{AsyncSession, TokioTcpStream};
use futures::AsyncReadExt;
use futures::AsyncSeekExt;
use futures::AsyncWriteExt;

use std::cmp::min;
use std::io::Error;
use std::io::SeekFrom;
use std::net::ToSocketAddrs;
use std::path::Path;
use tokio::io::AsyncReadExt as OtherAsyncReadExt;
use tokio::io::AsyncSeekExt as OtherAsyncSeekExt;

use crate::address::ParsedAddress;
use crate::bar::WrappedBar;
use crate::consts::*;
use crate::error::ValidateError;
use crate::hash::HashChecker;
use crate::io::get_output;
use crate::ssh_auth::get_possible_ssh_keys_path;

pub struct SFTPHandler;
impl SFTPHandler {
    pub async fn get(
        input: &str,
        output: &str,
        bar: &mut WrappedBar,
        expected_sha256: &str,
    ) -> Result<(), ValidateError> {
        SFTPHandler::_get(input, output, bar).await?;
        HashChecker::check(output, expected_sha256)
    }
    async fn _get(input: &str, output: &str, bar: &mut WrappedBar) -> Result<(), ValidateError> {
        let (session, remote_file) = SFTPHandler::setup_session(input, bar.silent).await;
        let (mut out, mut transferred) = get_output(output, bar.silent);
        let sftp = session.sftp().await.unwrap();
        let stat = sftp
            .stat(Path::new(&remote_file))
            .await
            .expect("Cannot stat remote SFTP file");
        let mut remote_file = sftp
            .open(Path::new(&remote_file))
            .await
            .expect("Cannot open remote SFTP file");
        let total_size = stat.size.expect("Cannot get remote SFTP file size");
        bar.set_length(total_size);

        remote_file
            .seek(SeekFrom::Current(transferred as i64))
            .await
            .expect("Cannot seek in SFTP file");
        loop {
            let mut buffer = vec![0; BUFFER_SIZE];
            let byte_count = remote_file
                .read(&mut buffer)
                .await
                .expect("Cannot read SFTP stream");
            buffer.truncate(byte_count);
            if !buffer.is_empty() {
                out.write_all(&buffer)
                    .map_err(|_| "Error while writing to output")
                    .unwrap();
                let new = min(transferred + (byte_count as u64), total_size);
                transferred = new;
                bar.set_position(new);
            } else {
                break;
            }
        }
        bar.finish_download(input, output);
        Ok(())
    }

    pub async fn put(input: &str, output: &str, mut bar: WrappedBar) -> Result<(), ValidateError> {
        let mut file = tokio::fs::File::open(&input)
            .await
            .expect("Cannot read input file");
        let total_size = file
            .metadata()
            .await
            .expect("Cannot determine input file length")
            .len();
        let (session, remote_file) = SFTPHandler::setup_session(output, bar.silent).await;
        let sftp = session.sftp().await.unwrap();
        let stat = sftp.stat(Path::new(&remote_file)).await;
        let (mut remote_file, mut transferred) = match stat {
            Ok(v) => (
                sftp.open(Path::new(&remote_file))
                    .await
                    .expect("Cannot open remote SFTP file"),
                v.size.expect("Cannot determine remote SFTP file size"),
            ),
            Err(_) => (sftp.create(Path::new(&remote_file)).await.unwrap(), 0),
        };
        bar.set_length(transferred);

        remote_file
            .seek(SeekFrom::Current(transferred as i64))
            .await
            .expect("Cannot seek in remote SFTP file");
        file.seek(SeekFrom::Current(transferred as i64))
            .await
            .expect("Cannot seek in local file");
        loop {
            let mut buffer = vec![0; BUFFER_SIZE];
            let byte_count = file
                .read(&mut buffer)
                .await
                .expect("Cannot read local file stream");
            buffer.truncate(byte_count);
            if !buffer.is_empty() {
                remote_file
                    .write_all(&buffer)
                    .await
                    .expect("Cannot write local file stream");
                let new = min(transferred + (byte_count as u64), total_size);
                transferred = new;
                bar.set_position(new);
            } else {
                break;
            }
        }
        bar.finish_download(input, output);

        Ok(())
    }
    async fn setup_session(address: &str, silent: bool) -> (AsyncSession<TokioTcpStream>, String) {
        let parsed_address = ParsedAddress::parse_address(address, silent);

        let addr = parsed_address
            .server
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        let stream = TokioTcpStream::connect(addr).await.unwrap();
        let mut session = AsyncSession::new(stream, None).unwrap();
        session.handshake().await.expect("SFTP handshake failed");
        if parsed_address.password != "anonymous" {
            session
                .userauth_password(&parsed_address.username, &parsed_address.password)
                .await
                .expect("SFTP Authentication failed");
        } else {
            let ssh_keys = get_possible_ssh_keys_path(silent);
            let mut is_ok = false;
            for ssh_key in ssh_keys.iter() {
                if session
                    .userauth_pubkey_file(
                        &parsed_address.username,
                        Some(Path::new(&(ssh_key.to_owned() + ".pub"))),
                        Path::new(ssh_key),
                        None,
                    )
                    .await
                    .is_ok()
                {
                    is_ok = true;
                    break;
                }
            }

            if !is_ok {
                println!("SFTP Authentication failed. Please specify a user: sftp://user@address");
            }
        }

        let remote_file = String::from("/")
            + &parsed_address.path_segments.join("/")[..]
            + "/"
            + &parsed_address.file[..];
        (session, remote_file)
    }

    pub async fn get_links(_input: String) -> Result<Vec<String>, Error> {
        panic!("Unimplemented");
    }
}

#[tokio::test]
#[should_panic]
async fn test_should_panic_when_not_implemented() {
    let _ = SFTPHandler::get_links("dummy".to_string()).await;
}
