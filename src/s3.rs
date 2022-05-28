extern crate http;
extern crate s3;

use question::Answer;
use question::Question;
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
use crate::tls;

struct Storage {
    _name: String,
    region: Region,
    credentials: Credentials,
    bucket: String,
    _location_supported: bool,
}

const MESSAGE: &str = "I want to go to S3";

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

        let bucket: &str = match parsed_address.path_segments.len() {
            0 => &parsed_address.file,
            _ => &parsed_address.path_segments[0],
        };

        let transport = S3::_get_transport(&parsed_address.server, AUTO_ALLOW_HTTP);
        let fqdn = transport.to_string() + &parsed_address.server;
        let bucket_kind = S3::_get_header(&fqdn, HTTP_HEADER_SERVER).await?;
        for backend in vec![S3::new(
            &bucket_kind,
            &parsed_address.username,
            &parsed_address.password,
            &bucket,
            &fqdn,
        )] {
            let bucket = Bucket::new(&backend.bucket, backend.region, backend.credentials)
                .unwrap()
                .with_path_style();

            let buckets = bucket.list("".to_string(), None).await.unwrap();
            for bucket in buckets {
                for content in bucket.contents {
                    println!("{}", content.key);
                }
            }

            let _ = S3::put_string(&bucket, "test_file", MESSAGE).await;
            let _ = S3::get_string(&bucket).await;
        }

        Ok(())
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

    fn _get_transport(server: &str, auto_allow_http: bool) -> &str {
        let parts: Vec<&str> = server.split(":").collect();
        let host = parts[0];
        let port = parts[1];
        if tls::has_tls(host, port) {
            return "https://";
        } else {
            if auto_allow_http
                || Question::new("Unsecure HTTP host. Continue? [Y/n]")
                    .default(Answer::YES)
                    .confirm()
                    == Answer::YES
            {
                return "http://";
            } else {
                return "";
            }
        }
    }
    async fn _get_binary(bucket: &Bucket) -> Result<(), S3Error> {
        let (data, code) = bucket.get_object("random.bin").await?;
        assert_eq!(code, http::StatusCode::OK);
        assert_eq!(data.len(), 3072);
        Ok(())
    }

    async fn _put_binary(bucket: &Bucket) -> Result<(), S3Error> {
        let random_bytes: Vec<u8> = (0..3072).map(|_| 33).collect();
        let (_, code) = bucket
            .put_object("random.bin", random_bytes.as_slice())
            .await?;
        assert_eq!(http::StatusCode::OK, code);
        Ok(())
    }

    async fn _get_tags(bucket: &Bucket) -> Result<(), S3Error> {
        let (tags, _status) = bucket.get_object_tagging("test_file").await?;
        println!("{:?}", tags);
        Ok(())
    }

    async fn _set_tags(bucket: &Bucket) -> Result<(), S3Error> {
        bucket
            .put_object_tagging("test_file", &[("test", "tag")])
            .await?;
        println!("Tags set");
        Ok(())
    }

    async fn _print_bucket_location(backend: Storage, bucket: &Bucket) -> Result<(), S3Error> {
        if backend._location_supported {
            // Get bucket location
            println!("{:?}", bucket.location().await?);
        }
        Ok(())
    }

    async fn put_string(
        bucket: &Bucket,
        destination_file: &str,
        string: &str,
    ) -> Result<(), S3Error> {
        let (_, code) = bucket.delete_object(destination_file).await?;
        assert_eq!(204, code);

        let (_, code) = bucket
            .put_object(destination_file, string.as_bytes())
            .await?;
        assert_eq!(http::StatusCode::OK, code);

        Ok(())
    }

    async fn get_string(bucket: &Bucket) -> Result<String, S3Error> {
        // Get the "test_file" contents and make sure that the returned message
        // matches what we sent.
        let (data, code) = bucket.get_object("test_file").await?;
        let string = str::from_utf8(&data)?;
        assert_eq!(http::StatusCode::OK, code);
        assert_eq!(MESSAGE, string);
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
                bucket: bucket.to_string(),
                _location_supported: false,
            },
            "aws" => Storage {
                _name: "aws".into(),
                region: "eu-central-1".parse().unwrap(),
                credentials: Credentials::from_env_specific(
                    Some(access_key),
                    Some(secret_key),
                    None,
                    None,
                )
                .unwrap(),
                bucket: bucket.to_string(),
                _location_supported: true,
            },
            "aws_public" => Storage {
                _name: "aws-public".into(),
                region: "eu-central-1".parse().unwrap(),
                credentials: Credentials::anonymous().unwrap(),
                bucket: bucket.to_string(),
                _location_supported: true,
            },
            _ => Storage {
                _name: "yandex".into(),
                region: "ru-central1".parse().unwrap(),
                credentials: Credentials::from_profile(Some("yandex")).unwrap(),
                bucket: bucket.to_string(),
                _location_supported: false,
            },
        };
        return storage;
    }
}
