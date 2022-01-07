extern crate ssh2;

use ssh2::Session;
use std::fs::File;
use std::net::TcpStream;
use std::path::Path;

use crate::address::ParsedAddress;
use crate::bar::WrappedBar;
use crate::hash::HashChecker;
use crate::io;
use crate::slicer::Slicer;

use indicatif::ProgressBar;

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
        let ssh_host_port = "127.0.0.1:22";
        let ssh_user = "mihai";
        let remote_temp_file = "/home/mihai/.vimrc";

        let tcp = TcpStream::connect(&ssh_host_port).unwrap();

        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().expect("SSH handshake failed");
        sess.userauth_password("demo", "demo12!").unwrap();

        let (remote_file, stat) = sess.scp_recv(Path::new(remote_temp_file)).unwrap();
        let stream: ssh2::Stream = remote_file.stream(1);

        let mut target = File::create("/tmp/done.txt").unwrap();
        let pb = ProgressBar::new(stat.size());
        std::io::copy(&mut pb.wrap_read(remote_file), &mut target).unwrap();

        //let (mut out, mut downloaded) = io::get_output(output, bar.silent);

        //   let res = Client::new()
        //       .get(input)
        //       .header("Range", "bytes=".to_owned() + &downloaded.to_string() + "-")
        //       .header(
        //           reqwest::header::USER_AGENT,
        //           reqwest::header::HeaderValue::from_static(CLIENT_ID),
        //       )
        //       .basic_auth(parsed_address.username, Some(parsed_address.password))
        //       .send()
        //       .await
        //       .or(Err(format!("Failed to GET from {} to {}", &input, &output)))
        //       .unwrap();
        //   let total_size = downloaded + res.content_length().or(Some(0)).unwrap();
        //
        //   bar.set_length(total_size);
        //
        //   let mut stream = res.bytes_stream();
        //   while let Some(item) = stream.next().await {
        //       let chunk = item.or(Err(format!("Error while downloading."))).unwrap();
        //       out.write_all(&chunk)
        //           .or(Err(format!("Error while writing to output.")))
        //           .unwrap();
        //       let new = min(downloaded + (chunk.len() as u64), total_size);
        //       downloaded = new;
        //       bar.set_position(new);
        //   }
        //
        //   bar.finish_download(&input, &output);
    }
}
