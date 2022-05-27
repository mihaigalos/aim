extern crate http;
extern crate s3;

use std::str;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::region::Region;

struct Storage {
    name: String,
    region: Region,
    credentials: Credentials,
    bucket: String,
    _location_supported: bool,
}

const MESSAGE: &str = "I want to go to S3";

pub struct S3;
impl S3 {
    pub async fn run(kind: &str, user: &str, pass: &str, bucket: &str) -> Result<(), S3Error> {
        for backend in vec![S3::new_storage(
            kind,
            user,
            pass,
            bucket,
            "http://172.17.0.2:9000",
        )] {
            println!("Running {}", backend.name);
            let bucket = Bucket::new(&backend.bucket, backend.region, backend.credentials)?
                .with_path_style();

            let buckets = bucket.list("".to_string(), None).await?;
            for bucket in buckets {
                println!("Bucket: {:?}", bucket.name);
                for content in bucket.contents {
                    println!("{:?}", content);
                }
            }
            println!("Done.");

            let _ = S3::put_string(&bucket, "test_file", MESSAGE).await;
            let _ = S3::get_string(&bucket).await;
        }

        Ok(())
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

    fn new_storage(
        kind: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
        endpoint: &str,
    ) -> Storage {
        let storage = match kind {
            "minio" => Storage {
                name: "minio".into(),
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
                name: "aws".into(),
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
                name: "aws-public".into(),
                region: "eu-central-1".parse().unwrap(),
                credentials: Credentials::anonymous().unwrap(),
                bucket: bucket.to_string(),
                _location_supported: true,
            },
            _ => Storage {
                name: "yandex".into(),
                region: "ru-central1".parse().unwrap(),
                credentials: Credentials::from_profile(Some("yandex")).unwrap(),
                bucket: bucket.to_string(),
                _location_supported: false,
            },
        };
        return storage;
    }
}
