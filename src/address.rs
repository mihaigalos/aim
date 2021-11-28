use failure::format_err;
use url::Url;

#[derive(Debug)]
pub struct FTPParsedAddress {
    pub server: String,
    pub username: String,
    pub password: String,
    pub path_segments: Vec<String>,
    pub file: String,
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

impl FTPParsedAddress {
    pub fn parse_ftp_address(address: &str) -> FTPParsedAddress {
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

    let actual = FTPParsedAddress::parse_ftp_address("ftp://user:pass@do.main:21/index/file");

    assert_eq!(actual, expected);
}
