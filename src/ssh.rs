extern crate ssh2;

use ssh2::Session;
use std::fs::File;
use std::net::TcpStream;
use std::path::Path;

use crate::address::ParsedAddress;
use crate::bar::WrappedBar;
use crate::hash::HashChecker;
use crate::slicer::Slicer;

pub struct SSHHandler;
impl SSHHandler {
    pub async fn get(
        input: &str,
        output: &str,
        bar: &mut WrappedBar,
        expected_sha256: &str,
    ) -> bool {
        let _output = match output {
            "." => Slicer::target_with_extension(input),
            _ => output,
        };
        SSHHandler::_get(input, _output, bar).await;
        HashChecker::check(_output, expected_sha256, bar.silent)
    }
    async fn _get(input: &str, output: &str, bar: &mut WrappedBar) {
        let ssh_host_port = "127.0.0.1:2222";
        let parsed_address = ParsedAddress::parse_address(input, bar.silent);
        let tcp = TcpStream::connect(&ssh_host_port).unwrap();
        let mut sess = Session::new().unwrap();

        sess.set_tcp_stream(tcp);
        sess.handshake().expect("SSH handshake failed");
        if parsed_address.password != "" {
            sess.userauth_password(&parsed_address.username, &parsed_address.password)
                .unwrap();
        } else {
            sess.userauth_password(&parsed_address.username, "")
                .unwrap();
        }
        let remote_file = &(String::from("/")
            + &parsed_address.path_segments.join("/")
            + "/"
            + &parsed_address.file);

        let (remote_file_contents, stat) = sess.scp_recv(Path::new(remote_file)).unwrap();
        let _: ssh2::Stream = remote_file_contents.stream(1);

        let mut target = File::create(output).unwrap();
        bar.set_length(stat.size());
        std::io::copy(
            &mut bar.output.as_ref().unwrap().wrap_read(remote_file_contents),
            &mut target,
        )
        .unwrap();
    }
}
