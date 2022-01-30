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
        let parsed_address = ParsedAddress::parse_address(input, bar.silent);
        let tcp =
            TcpStream::connect(&parsed_address.server).expect("Cannot connect to SSH address.");
        let mut sess = Session::new().unwrap();

        sess.set_tcp_stream(tcp);
        sess.handshake().expect("SSH handshake failed.");
        if parsed_address.password != "" {
            sess.userauth_password(&parsed_address.username, &parsed_address.password)
                .expect("SSH Authentication failed.");
        } else {
            sess.userauth_password(&parsed_address.username, "")
                .expect("SSH Authentication failed. No password specified. Is passwordless authentication set up?");
        }
        let remote_file = &(String::from("/")
            + &parsed_address.path_segments.join("/")
            + "/"
            + &parsed_address.file);

        let (remote_file_contents, stat) = sess
            .scp_recv(Path::new(remote_file))
            .expect(&format!("Remove file does not exist: {}", input));

        let mut target = File::create(output).unwrap();
        bar.set_length(stat.size());
        std::io::copy(
            &mut bar.output.as_ref().unwrap().wrap_read(remote_file_contents),
            &mut target,
        )
        .unwrap();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn just_start(justfile: &str) {
        use std::env;
        use std::io::{self, Write};
        use std::process::Command;
        let output = Command::new("just")
            .args([
                "--justfile",
                justfile,
                "_start",
                env::current_dir().unwrap().to_str().unwrap(),
            ])
            .output()
            .expect("failed to just _start");

        println!("status: {}", output.status);
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }

    fn just_stop(justfile: &str) {
        use std::env;
        use std::process::Command;
        let _ = Command::new("just")
            .args([
                "--justfile",
                justfile,
                "_stop",
                env::current_dir().unwrap().to_str().unwrap(),
            ])
            .output();
    }

    #[tokio::test]
    #[serial]
    async fn test_ssh_get_works_when_typical() {
        //just_start("test/ssh/Justfile");

        let result = SSHHandler::get(
            "ssh://user:pass@127.0.0.1:2222/tmp/binfile",
            "_test_ssh_get_works_when_typical",
            &mut WrappedBar::new_empty(),
            "",
        )
        .await;

        assert!(result);

        //just_stop("test/https/Justfile");
    }
}
