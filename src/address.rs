use failure::format_err;
use url::Url;

#[derive(Debug)]
pub struct ParsedAddress {
    pub server: String,
    pub username: String,
    pub password: String,
    pub path_segments: Vec<String>,
    pub file: String,
}

impl PartialEq for ParsedAddress {
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

impl ParsedAddress {
    pub fn parse_address(address: &str, netrc: Option<netrc::Netrc>) -> ParsedAddress {
        let url = Url::parse(address).unwrap();
        let server = format!(
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

        let (username, password) = ParsedAddress::mixin_netrc(&netrc, &server, username, password);

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

        ParsedAddress {
            server: server,
            username: username,
            password: password,
            path_segments: path_segments,
            file: file.to_string(),
        }
    }

    fn mixin_netrc(
        netrc: &Option<netrc::Netrc>,
        server: &str,
        username: String,
        password: String,
    ) -> (String, String) {
        let mut user = username.clone();
        let mut pass = password.clone();
        if !netrc.is_none() && username == "anonymous" && password == "anonymous" {
            for host in netrc.as_ref().unwrap().hosts.iter().enumerate() {
                let (_i, (netrc_name, machine)) = host;

                let mut name = netrc_name.to_string();
                if let Some(port) = machine.port {
                    name = name + ":" + &port.to_string();
                }
                if server == name {
                    user = machine.login.clone();
                    pass = machine.password.clone().unwrap();
                    break;
                }
            }
        }
        (user, pass)
    }
}

#[tokio::test]
async fn parseaddress_operator_equals_works_when_typical() {
    let left = ParsedAddress {
        server: "do.main".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["my".to_string(), "path".to_string()],
        file: "pass".to_string(),
    };
    let right = ParsedAddress {
        server: "do.main".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["my".to_string(), "path".to_string()],
        file: "pass".to_string(),
    };

    assert!(left == right);
}

#[tokio::test]
async fn parseaddress_operator_equals_fails_when_not_equal() {
    let left = ParsedAddress {
        server: "do.main".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["my".to_string(), "path".to_string()],
        file: "pass".to_string(),
    };
    let right = ParsedAddress {
        server: "do".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["my".to_string(), "path".to_string()],
        file: "pass".to_string(),
    };

    assert!(left != right);
}

#[tokio::test]
async fn parse_works() {
    let expected = ParsedAddress {
        server: "do.main:21".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        path_segments: vec!["index".to_string()],
        file: "file".to_string(),
    };

    let actual = ParsedAddress::parse_address("ftp://user:pass@do.main:21/index/file", None);

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn mixin_works() {
    let expected_username = "test";
    let expected_password = "p@ssw0rd";
    let input = "machine example.com login test password p@ssw0rd";
    let input = std::io::BufReader::new(input.as_bytes());
    let netrc = netrc::Netrc::parse(input).unwrap();
    let username_decoded_from_url = "anonymous".to_string();
    let password_decoded_from_url = "anonymous".to_string();

    let (actual_username, actual_password) = ParsedAddress::mixin_netrc(
        &Some(netrc),
        "example.com",
        username_decoded_from_url,
        password_decoded_from_url,
    );

    assert_eq!(actual_username, expected_username);
    assert_eq!(actual_password, expected_password);
}

#[tokio::test]
async fn mixin_works_with_port() {
    let expected_username = "test";
    let expected_password = "p@ssw0rd";
    let input = "machine example.com login test password p@ssw0rd port 443";
    let input = std::io::BufReader::new(input.as_bytes());
    let netrc = netrc::Netrc::parse(input).unwrap();
    let username_decoded_from_url = "anonymous".to_string();
    let password_decoded_from_url = "anonymous".to_string();

    let (actual_username, actual_password) = ParsedAddress::mixin_netrc(
        &Some(netrc),
        "example.com:443",
        username_decoded_from_url,
        password_decoded_from_url,
    );

    assert_eq!(actual_username, expected_username);
    assert_eq!(actual_password, expected_password);
}
#[tokio::test]
async fn parse_works_with_netrc_mixin() {
    let expected = ParsedAddress {
        server: "do.main:21".to_string(),
        username: "test".to_string(),
        password: "p@ssw0rd".to_string(),
        path_segments: vec!["index".to_string()],
        file: "file".to_string(),
    };
    let input = "machine do.main login test password p@ssw0rd port 21";
    let input = std::io::BufReader::new(input.as_bytes());
    let netrc = netrc::Netrc::parse(input).unwrap();
    let actual = ParsedAddress::parse_address("ftp://do.main/index/file", Some(netrc));

    assert_eq!(actual, expected);
}
