extern crate ssh2;

use ssh2::Session;
use std::fs::File;
use std::net::TcpStream;
use std::path::Path;

use crate::address::ParsedAddress;
use crate::bar::WrappedBar;
use crate::error::ValidateError;
use crate::hash::HashChecker;
use crate::slicer::Slicer;

pub struct SSHHandler;
impl SSHHandler {
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
        SSHHandler::_get(input, _output, bar).await;
        HashChecker::check(_output, expected_sha256, bar.silent)
    }
    async fn _get(input: &str, output: &str, bar: &mut WrappedBar) {
        let parsed_address = ParsedAddress::parse_address(input, bar.silent);
        let tcp =
            TcpStream::connect(&parsed_address.server).expect("Cannot connect to SSH address");
        let mut session = Session::new().unwrap();

        session.set_tcp_stream(tcp);
        session.handshake().expect("SSH handshake failed");
        if parsed_address.password != "" {
            session
                .userauth_password(&parsed_address.username, &parsed_address.password)
                .expect("SSH Authentication failed");
        } else {
            session.userauth_password(&parsed_address.username, "")
                .expect("SSH Authentication failed. No password specified. Is passwordless authentication set up?");
        }
        let remote_file = &(String::from("/")
            + &parsed_address.path_segments.join("/")
            + "/"
            + &parsed_address.file);

        let (channel, stat) = session
            .scp_recv(Path::new(remote_file))
            .expect(&format!("Remove file does not exist: {}", input));

        let mut target =
            File::create(output).expect(&format!("Cannot create output file: {}", output));
        bar.set_length(stat.size());

        std::io::copy(
            &mut bar.output.as_ref().unwrap().wrap_read(channel),
            &mut target,
        )
        .expect("Cannot write contents to file");
    }

    pub async fn put(input: &str, output: &str, mut bar: WrappedBar) -> Result<(), ValidateError> {
        let (session, remote_file) = SSHHandler::setup_session(output, bar.silent);
        let input_file = File::open(&input).expect("Cannot open input file for SSH read");
        let total_size = input_file
            .metadata()
            .expect("Cannot determine input file size for HTTPS read")
            .len();

        let mut channel = session
            .scp_send(Path::new(&remote_file), 0o644, total_size, None)
            .expect(&format!("Cannot create SSH channel"));

        bar.set_length(total_size);

        std::io::copy(
            &mut bar.output.as_ref().unwrap().wrap_read(input_file),
            &mut channel,
        )
        .expect("Cannot write contents to file");
        Ok(())
    }

    fn setup_session(output: &str, silent: bool) -> (Session, String) {
        let parsed_address = ParsedAddress::parse_address(output, silent);
        let tcp =
            TcpStream::connect(&parsed_address.server).expect("Cannot connect to SSH address");
        let mut session = Session::new().unwrap();

        session.set_tcp_stream(tcp);
        session.handshake().expect("SSH handshake failed");
        if parsed_address.password != "" {
            session
                .userauth_password(&parsed_address.username, &parsed_address.password)
                .expect("SSH Authentication failed");
        } else {
            session.userauth_password(&parsed_address.username, "")
                .expect("SSH Authentication failed. No password specified. Is passwordless authentication set up?");
        }
        let remote_file = String::from("/")
            + &parsed_address.path_segments.join("/")
            + "/"
            + &parsed_address.file;
        (session, remote_file)
    }
}
