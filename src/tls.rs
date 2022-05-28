extern crate native_tls;

use native_tls::TlsConnector;
use std::net::TcpStream;

pub fn has_tls(host: &str, port: &str) -> bool {
    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect(host.to_string() + ":" + port).unwrap();
    let stream = connector.connect(host, stream).ok();
    stream.is_some()
}

#[test]
fn test_has_tls_when_typical() {
    assert_eq!(has_tls("google.com", "443"), true);
}

#[test]
fn test_has_tls_when_no_tls() {
    assert_eq!(has_tls("localhost", "9000"), false);
}
