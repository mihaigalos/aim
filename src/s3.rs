extern crate http;
extern crate s3;

use std::io::Error;
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

    pub async fn put(input: &str, output: &str, bar: WrappedBar) -> Result<(), ValidateError> {
        let (output, bucket) = S3::setup(output, bar.silent).await;

        let mut async_input_file = tokio::fs::File::open(input) //TODO: when s3 provider crate has stream support implementing futures_core::stream::Stream used in resume, use io.rs::get_output() instead.
            .await
            .expect("Unable to open input file");

        let _ = bucket
            .put_object_stream(&mut async_input_file, output)
            .await
            .unwrap();
        Ok(())
    }

    async fn _get(input: &str, output: &str, bar: &mut WrappedBar) -> Result<(), HTTPHeaderError> {
        let (input, bucket) = S3::setup(input, bar.silent).await;
        let mut async_output_file = tokio::fs::File::create(output) //TODO: when s3 provider crate has stream support implementing futures_core::stream::Stream used in resume, use io.rs::get_output() instead.
            .await
            .expect("Unable to open output file");
        let _ = bucket
            .get_object_stream(input, &mut async_output_file)
            .await
            .unwrap();
        Ok(())
    }

    async fn setup(io: &str, silent: bool) -> (String, s3::bucket::Bucket) {
        let parsed_address = ParsedAddress::parse_address(io, silent);
        let io = S3::get_path_in_bucket(&parsed_address);
        let bucket = S3::get_bucket(&parsed_address);
        let transport = S3::_get_transport::<TLS, QuestionWrapped>(&parsed_address.server);
        let fqdn = transport.to_string() + &parsed_address.server;
        let bucket_kind = S3::_get_header(&fqdn, HTTP_HEADER_SERVER).await.unwrap();
        let (username, password) = S3::get_credentials(&parsed_address, silent);
        let backend = S3::new(&bucket_kind, &username, &password, &bucket, &fqdn);
        let bucket = Bucket::new(bucket, backend.region, backend.credentials)
            .unwrap()
            .with_path_style();
        (io, bucket)
    }

    fn get_credentials(parsed_address: &ParsedAddress, silent: bool) -> (String, String) {
        let result = (
            parsed_address.username.to_owned(),
            parsed_address.password.to_owned(),
        );

        let result = S3::mixin_aws_credentials_from_aws_folder(result.0, result.1, silent);
        let result = S3::mixin_aws_credentials_from_env(result.0, result.1, silent);
        (result.0, result.1)
    }

    fn mixin_aws_credentials_from_aws_folder(
        username: String,
        password: String,
        silent: bool,
    ) -> (String, String) {
        let mut result = (username, password);
        if let Ok(creds_from_profile) = Credentials::from_profile(None) {
            if !silent {
                println!("🔑 Parsed AWS credentials from ~/.aws/credentials.");
            }
            result = (
                creds_from_profile.access_key.unwrap(),
                creds_from_profile.secret_key.unwrap(),
            );
        }
        return result;
    }

    fn mixin_aws_credentials_from_env(
        username: String,
        password: String,
        silent: bool,
    ) -> (String, String) {
        let mut result = (username, password);
        if let Ok(creds_from_profile) = Credentials::from_env() {
            if !silent {
                println!("🔑 Parsed AWS credentials from environment vars AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY.");
            }

            result = (
                creds_from_profile.access_key.unwrap(),
                creds_from_profile.secret_key.unwrap(),
            );
        }
        return result;
    }

    fn get_path_in_bucket(parsed_address: &ParsedAddress) -> String {
        let mut result = "/".to_string();
        if parsed_address.path_segments.len() > 1 {
            result += &parsed_address.path_segments[1..].join("/");
            result += "/";
        }
        result += &parsed_address.file;
        return result;
    }

    fn get_bucket(parsed_address: &ParsedAddress) -> &str {
        let bucket: &str = match parsed_address.path_segments.len() {
            0 => &parsed_address.file,
            _ => &parsed_address.path_segments[0],
        };
        bucket
    }

    async fn _list(bucket: &Bucket) -> Result<Vec<String>, S3Error> {
        let mut result: Vec<String> = Vec::new();

        let pages = bucket.list("".to_string(), None).await?;
        for page in pages {
            for content in page.contents {
                result.push(content.key);
            }
        }
        Ok(result)
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
        assert_eq!(parts.len(), 2, "No port in URL. Stopping.");
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

    async fn _get_string(bucket: &Bucket, source_file: &str) -> Result<String, S3Error> {
        let (data, _) = bucket.get_object(source_file).await?;
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

    pub async fn get_links(_input: String) -> Result<Vec<String>, Error> {
        panic!("Unimplemented");
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
    async fn test_list_bucket_works_when_typical() {
        just_start("test/s3/Justfile");

        let parsed_address = ParsedAddress {
            server: "localhost:9000".to_string(),
            username: "minioadmin".to_string(),
            password: "minioadmin".to_string(),
            path_segments: vec!["test-bucket".to_string()],
            file: "".to_string(),
        };
        let bucket = S3::get_bucket(&parsed_address);

        let transport = S3::_get_transport::<TLS, QuestionWrapped>(&parsed_address.server);
        let fqdn = transport.to_string() + &parsed_address.server;
        let bucket_kind = S3::_get_header(&fqdn, HTTP_HEADER_SERVER).await.unwrap();
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

        assert!(S3::_list(&bucket).await.is_ok());

        just_stop("test/s3/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_put_string_works_when_typical() {
        just_start("test/s3/Justfile");

        let parsed_address = ParsedAddress {
            server: "localhost:9000".to_string(),
            username: "minioadmin".to_string(),
            password: "minioadmin".to_string(),
            path_segments: vec!["test-bucket".to_string()],
            file: "".to_string(),
        };
        let bucket = S3::get_bucket(&parsed_address);

        let transport = S3::_get_transport::<TLS, QuestionWrapped>(&parsed_address.server);
        let fqdn = transport.to_string() + &parsed_address.server;
        let bucket_kind = S3::_get_header(&fqdn, HTTP_HEADER_SERVER).await.unwrap();
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

        assert!(S3::_put_string(
            &bucket,
            "test_put_string_works_when_typical",
            "This is the string from test_put_string_works_when_typical."
        )
        .await
        .is_ok());

        just_stop("test/s3/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_string_works_when_typical() {
        just_start("test/s3/Justfile");

        let parsed_address = ParsedAddress {
            server: "localhost:9000".to_string(),
            username: "minioadmin".to_string(),
            password: "minioadmin".to_string(),
            path_segments: vec!["test-bucket".to_string()],
            file: "".to_string(),
        };
        let bucket = S3::get_bucket(&parsed_address);

        let transport = S3::_get_transport::<TLS, QuestionWrapped>(&parsed_address.server);
        let fqdn = transport.to_string() + &parsed_address.server;
        let bucket_kind = S3::_get_header(&fqdn, HTTP_HEADER_SERVER).await.unwrap();
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

        let _ = S3::_put_string(
            &bucket,
            "test_put_string_works_when_typical",
            "This is the string from test_put_string_works_when_typical.",
        )
        .await
        .is_ok();

        assert_eq!(
            S3::_get_string(&bucket, "test_put_string_works_when_typical")
                .await
                .unwrap(),
            "This is the string from test_put_string_works_when_typical."
        );

        just_stop("test/s3/Justfile");
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

    #[should_panic]
    #[tokio::test]
    async fn test_get_transport_bucket_panics_when_no_port() {
        let parsed_address = ParsedAddress {
            server: "localhost".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            path_segments: vec!["test-bucket".to_string()],
            file: "".to_string(),
        };
        let _ = S3::_get_transport::<TLS, QuestionWrapped>(&parsed_address.server);
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

    #[test]
    fn test_get_path_in_bucket_works_when_typical() {
        let parsed_address = ParsedAddress {
            server: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            path_segments: vec!["test-bucket".to_string()],
            file: "test-file".to_string(),
        };
        let path = S3::get_path_in_bucket(&parsed_address);
        assert_eq!(path, "/test-file");
    }
    #[test]
    fn test_get_path_in_bucket_works_when_full_url() {
        let parsed_address = ParsedAddress::parse_address(
            "s3://minioadmin:minioadmin@localhost:9000/test-bucket/test.file",
            true,
        );
        let path = S3::get_path_in_bucket(&parsed_address);
        assert_eq!(path, "/test.file");
    }

    #[test]
    fn test_get_path_in_bucket_works_when_in_subfolder() {
        let parsed_address = ParsedAddress::parse_address(
            "s3://minioadmin:minioadmin@localhost:9000/test-bucket/subfolder/test.file",
            true,
        );
        let path = S3::get_path_in_bucket(&parsed_address);
        assert_eq!(path, "/subfolder/test.file");
    }

    #[test]
    fn test_get_credentials_works_when_tyipical() {
        let parsed_address = ParsedAddress::parse_address(
            "s3://user:pass@localhost:9000/test-bucket/subfolder/test.file",
            true,
        );

        let (username, password) = S3::get_credentials(&parsed_address, true);

        assert_eq!(
            (username, password),
            ("user".to_string(), "pass".to_string())
        )
    }

    #[test]
    fn test_get_credentials_works_when_tyipical_and_not_silent() {
        let parsed_address = ParsedAddress::parse_address(
            "s3://user:pass@localhost:9000/test-bucket/subfolder/test.file",
            true,
        );

        let (username, password) = S3::get_credentials(&parsed_address, false);

        assert_eq!(
            (username, password),
            ("user".to_string(), "pass".to_string())
        )
    }

    #[tokio::test]
    #[should_panic]
    async fn test_should_panic_when_not_implemented() {
        let _ = S3::get_links("dummy".to_string()).await;
    }
}

#[cfg(test)]
mod test_mixins {
    use super::*;
    use serial_test::serial;
    #[test]
    #[serial]
    fn test_mixin_aws_credentials_from_aws_folder_works_when_typical() {
        use std::fs::OpenOptions;
        use std::io::Write;
        use untildify::untildify;

        let _ = std::fs::rename(untildify("~/.aws"), untildify("~/.aws_aim_testing"));

        std::fs::create_dir(untildify("~/.aws")).unwrap();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(untildify("~/.aws/credentials"))
            .unwrap();
        file.write_all(b"[default]\n").unwrap();
        file.write_all(b"aws_access_key_id = credentials_user\n")
            .unwrap();
        file.write_all(b"aws_secret_access_key = credentials_pass")
            .unwrap();

        let (username, password) =
            S3::mixin_aws_credentials_from_aws_folder("".to_string(), "".to_string(), true);

        std::fs::remove_dir_all(untildify("~/.aws")).unwrap();
        let _ = std::fs::rename(untildify("~/.aws_aim_testing"), untildify("~/.aws"));
        assert_eq!(
            (username, password),
            (
                "credentials_user".to_string(),
                "credentials_pass".to_string()
            )
        );
    }
    #[test]
    #[serial]
    fn test_mixin_aws_credentials_from_aws_folder_works_when_typical_and_not_silent() {
        use std::fs::OpenOptions;
        use std::io::Write;
        use untildify::untildify;

        let _ = std::fs::rename(untildify("~/.aws"), untildify("~/.aws_aim_testing"));

        std::fs::create_dir(untildify("~/.aws")).unwrap();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(untildify("~/.aws/credentials"))
            .unwrap();
        file.write_all(b"[default]\n").unwrap();
        file.write_all(b"aws_access_key_id = credentials_user\n")
            .unwrap();
        file.write_all(b"aws_secret_access_key = credentials_pass")
            .unwrap();

        let (username, password) =
            S3::mixin_aws_credentials_from_aws_folder("".to_string(), "".to_string(), false);

        std::fs::remove_dir_all(untildify("~/.aws")).unwrap();
        let _ = std::fs::rename(untildify("~/.aws_aim_testing"), untildify("~/.aws"));
        assert_eq!(
            (username, password),
            (
                "credentials_user".to_string(),
                "credentials_pass".to_string()
            )
        );
    }

    #[test]
    #[serial]
    fn test_mixin_aws_credentials_from_env_works_when_typical() {
        use std::env;
        let old_access_key = env::var("AWS_ACCESS_KEY_ID").unwrap_or("".to_string());
        let old_secret_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap_or("".to_string());
        env::set_var("AWS_ACCESS_KEY_ID", "myaccesskey");
        env::set_var("AWS_SECRET_ACCESS_KEY", "mysecretkey");

        let (username, password) =
            S3::mixin_aws_credentials_from_env("".to_string(), "".to_string(), true);

        env::set_var("AWS_ACCESS_KEY_ID", old_access_key);
        env::set_var("AWS_SECRET_ACCESS_KEY", old_secret_key);

        assert_eq!(
            (username, password),
            ("myaccesskey".to_string(), "mysecretkey".to_string())
        );
    }
    #[test]
    #[serial]
    fn test_mixin_aws_credentials_from_env_works_when_typical_and_not_silent() {
        use std::env;
        let old_access_key = env::var("AWS_ACCESS_KEY_ID").unwrap_or("".to_string());
        let old_secret_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap_or("".to_string());
        env::set_var("AWS_ACCESS_KEY_ID", "myaccesskey");
        env::set_var("AWS_SECRET_ACCESS_KEY", "mysecretkey");

        let (username, password) =
            S3::mixin_aws_credentials_from_env("".to_string(), "".to_string(), false);

        env::set_var("AWS_ACCESS_KEY_ID", old_access_key);
        env::set_var("AWS_SECRET_ACCESS_KEY", old_secret_key);

        assert_eq!(
            (username, password),
            ("myaccesskey".to_string(), "mysecretkey".to_string())
        );
    }
}
