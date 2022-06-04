extern crate http;
extern crate s3;

use std::str;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::region::Region;

use crate::address::ParsedAddress;
use crate::bar::WrappedBar;
use crate::consts::*;
use crate::error::HTTPHeaderError;
use crate::error::ValidateError;
use crate::hash::HashChecker;
use crate::io;
use crate::question::*;
use crate::tls::*;

struct Storage {
    _name: String,
    region: Region,
    credentials: Credentials,
    _bucket: String,
    _location_supported: bool,
}

pub struct S3;
impl S3 {
    pub async fn get(
        input: &str,
        output: &str,
        bar: &mut WrappedBar,
        expected_sha256: &str,
    ) -> Result<(), ValidateError> {
        S3::_get(input, output, bar).await.unwrap();
        HashChecker::check(output, expected_sha256)
    }

    async fn _get(input: &str, output: &str, bar: &mut WrappedBar) -> Result<(), HTTPHeaderError> {
        let parsed_address = ParsedAddress::parse_address(input, bar.silent);
        let (_, _) = io::get_output(output, bar.silent);

        let bucket = S3::get_bucket(&parsed_address);

        let transport = S3::_get_transport::<TLS, QuestionWrapped>(&parsed_address.server);
        let fqdn = transport.to_string() + &parsed_address.server;
        let bucket_kind = S3::_get_header(&fqdn, HTTP_HEADER_SERVER).await?;
        let backend = S3::new(
            &bucket_kind,
            &parsed_address.username,
            &parsed_address.password,
            &bucket,
            &fqdn,
        );

        let bucket = Bucket::new(bucket, backend.region, backend.credentials)
            .unwrap()
            .with_path_style();

        let buckets = bucket.list("".to_string(), None).await.unwrap();
        for bucket in buckets {
            for content in bucket.contents {
                println!("{}", content.key);
            }
        }

        Ok(())
    }

    fn get_bucket(parsed_address: &ParsedAddress) -> &str {
        let bucket: &str = match parsed_address.path_segments.len() {
            0 => &parsed_address.file,
            _ => &parsed_address.path_segments[0],
        };
        bucket
    }

    async fn _get_header(server: &str, header: &str) -> Result<String, HTTPHeaderError> {
        let client = reqwest::Client::new();
        let res = client.post(server).send().await.unwrap();

        let result = res
            .headers()
            .get(header)
            .ok_or(HTTPHeaderError::NotPresent)?;

        Ok(result.to_str().unwrap().to_lowercase().to_string())
    }

    fn _get_transport<T: TLSTrait, Q: QuestionTrait>(server: &str) -> &str {
        let parts: Vec<&str> = server.split(":").collect();
        let host = parts[0];
        let port = parts[1];
        if T::has_tls(host, port) {
            return "https://";
        } else {
            if Q::yes_no() {
                return "http://";
            } else {
                return "";
            }
        }
    }

    async fn _put_string(
        bucket: &Bucket,
        destination_file: &str,
        string: &str,
    ) -> Result<(), S3Error> {
        let (_, _) = bucket.delete_object(destination_file).await?;
        let (_, _) = bucket
            .put_object(destination_file, string.as_bytes())
            .await?;

        Ok(())
    }

    async fn _get_string(bucket: &Bucket) -> Result<String, S3Error> {
        let (data, _) = bucket.get_object("test_file").await?;
        let string = str::from_utf8(&data)?;
        Ok(string.to_string())
    }

    fn new(
        kind: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
        endpoint: &str,
    ) -> Storage {
        let storage = match kind {
            "minio" => Storage {
                _name: "minio".into(),
                region: Region::Custom {
                    region: "".into(),
                    endpoint: endpoint.into(),
                },
                credentials: Credentials {
                    access_key: Some(access_key.to_owned()),
                    secret_key: Some(secret_key.to_owned()),
                    security_token: None,
                    session_token: None,
                },
                _bucket: bucket.to_string(),
                _location_supported: false,
            },
            "aws" => Storage {
                _name: "aws".into(),
                region: "eu-central-1".parse().unwrap(),
                credentials: Credentials {
                    access_key: Some(access_key.to_owned()),
                    secret_key: Some(secret_key.to_owned()),
                    security_token: None,
                    session_token: None,
                },
                _bucket: bucket.to_string(),
                _location_supported: true,
            },
            _ => Storage {
                _name: "".into(),
                region: "".parse().unwrap(),
                credentials: Credentials {
                    access_key: Some(access_key.to_owned()),
                    secret_key: Some(secret_key.to_owned()),
                    security_token: None,
                    session_token: None,
                },
                _bucket: bucket.to_string(),
                _location_supported: false,
            },
        };
        return storage;
    }
}
#[test]
fn test_get_bucket_works_when_typical() {
    let parsed_address = ParsedAddress {
        server: "".to_string(),
        username: "".to_string(),
        password: "".to_string(),
        path_segments: vec!["test-bucket".to_string()],
        file: "".to_string(),
    };
    assert_eq!(S3::get_bucket(&parsed_address), "test-bucket");
}

#[test]
fn test_get_bucket_works_when_multiple_segments() {
    let parsed_address = ParsedAddress {
        server: "".to_string(),
        username: "".to_string(),
        password: "".to_string(),
        path_segments: vec!["test-bucket".to_string(), "test-file".to_string()],
        file: "".to_string(),
    };
    assert_eq!(S3::get_bucket(&parsed_address), "test-bucket");
}

#[test]
fn test_get_transport_returns_http_transport_when_no_tls() {
    use crate::question::*;
    pub struct TlsMockNoTLS;
    impl TLSTrait for TlsMockNoTLS {
        fn has_tls(_host: &str, _port: &str) -> bool {
            false
        }
    }
    assert_eq!(
        S3::_get_transport::<TlsMockNoTLS, QuestionWrapped>("dummyhost:9000"),
        "http://"
    );
}

#[test]
fn test_get_transport_returns_https_transport_when_has_tls() {
    use crate::question::*;
    pub struct TlsMockHasTLS;
    impl TLSTrait for TlsMockHasTLS {
        fn has_tls(_host: &str, _port: &str) -> bool {
            true
        }
    }
    assert_eq!(
        S3::_get_transport::<TlsMockHasTLS, QuestionWrapped>("dummyhost:9000"),
        "https://"
    );
}

#[test]
fn test_get_transport_returns_no_transport_when_no_tls() {
    use crate::question::*;
    pub struct TlsMockHasTLS;
    impl TLSTrait for TlsMockHasTLS {
        fn has_tls(_host: &str, _port: &str) -> bool {
            false
        }
    }
    struct QuestionWrappedMock;
    impl QuestionTrait for QuestionWrappedMock {
        fn yes_no() -> bool {
            false
        }
    }
    assert_eq!(
        S3::_get_transport::<TlsMockHasTLS, QuestionWrappedMock>("dummyhost:9000"),
        ""
    );
}

#[test]
fn test_storage_new_minio() {
    let storage = S3::new("minio", "user", "pass", "bucket", "fqdn");
    assert_eq!(storage._location_supported, false);
}

#[test]
fn test_storage_new_aws() {
    let storage = S3::new("aws", "user", "pass", "bucket", "fqdn");
    assert_eq!(storage._location_supported, true);
}

#[test]
fn test_storage_new_default() {
    let storage = S3::new("unknown", "user", "pass", "bucket", "fqdn");
    assert_eq!(storage._location_supported, false);
}
