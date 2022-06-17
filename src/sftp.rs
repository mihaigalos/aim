extern crate ssh2;

use async_io::Async;
use async_ssh2_lite::AsyncSession;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::address::ParsedAddress;
use crate::bar::WrappedBar;
use crate::error::ValidateError;
use crate::hash::HashChecker;
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
        let (session, remote_file) = SFTPHandler::setup_session(output, bar.silent).await;

        let filename = PathBuf::from("/tmp").join(Uuid::new_v4().to_string());
        let filename = filename.as_path();

        session.create(filename).await.unwrap();
        let file_stat = session.stat(filename).await.unwrap();
        println!("file_stat: {:?}", file_stat);

        session.unlink(filename).await.unwrap();

        println!("done");

        Ok(())
    }

    // pub async fn put(input: &str, output: &str, mut bar: WrappedBar) -> Result<(), ValidateError> {}
    async fn setup_session(
        address: &str,
        silent: bool,
    ) -> (async_ssh2_lite::AsyncSftp<std::net::TcpStream>, String) {
        let parsed_address = ParsedAddress::parse_address(address, silent);

        let addr = parsed_address
            .server
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        let stream = Async::<TcpStream>::connect(addr).await.unwrap();
        let mut session = AsyncSession::new(stream, None).unwrap();
        session.handshake().await.unwrap();
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
                println!("SFTP Authentication failed. No password specified. Is passwordless authentication set up?");
            }
        }

        let remote_file = String::from("/")
            + &parsed_address.path_segments.join("/")
            + "/"
            + &parsed_address.file;
        (session.sftp().await.unwrap(), remote_file)
    }
}
