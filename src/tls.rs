extern crate native_tls;

use native_tls::TlsConnector;
use std::net::TcpStream;

pub trait TLSTrait {
    fn has_tls(host: &str, port: &str) -> bool;
}

pub struct TLS;
impl TLSTrait for TLS {
    fn has_tls(host: &str, port: &str) -> bool {
        let connector = TlsConnector::new().unwrap();
        let stream = TcpStream::connect(host.to_string() + ":" + port).unwrap();
        let stream = connector.connect(host, stream).ok();
        stream.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_has_tls_when_typical() {
        assert_eq!(TLS::has_tls("google.com", "443"), true);
    }
}
