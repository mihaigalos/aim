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
    location_supported: bool,
}

const MESSAGE: &str = "I want to go to S3";

pub struct S3;
impl S3 {
    pub async fn run() -> Result<(), S3Error> {
        for backend in vec![S3::new_storage("minio")] {
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

            // Make sure that our "test_file" doesn't exist, delete it if it does. Note
            // that the s3 library returns the HTTP code even if it indicates a failure
            // (i.e. 404) since we can't predict desired usage. For example, you may
            // expect a 404 to make sure a fi le doesn't exist.
            //    let (_, code) = bucket.delete("test_file")?;
            //    assert_eq!(204, code);

            // Put a "test_file" with the contents of MESSAGE at the root of the
            // bucket.
            let (_, code) = bucket.put_object("test_file", MESSAGE.as_bytes()).await?;
            // println!("{}", bucket.presign_get("test_file", 604801, None)?);
            assert_eq!(http::StatusCode::OK, code);

            // Get the "test_file" contents and make sure that the returned message
            // matches what we sent.
            let (data, code) = bucket.get_object("test_file").await?;
            let string = str::from_utf8(&data)?;
            // println!("{}", string);
            assert_eq!(http::StatusCode::OK, code);
            assert_eq!(MESSAGE, string);

            if backend.location_supported {
                // Get bucket location
                println!("{:?}", bucket.location().await?);
            }

            bucket
                .put_object_tagging("test_file", &[("test", "tag")])
                .await?;
            println!("Tags set");
            let (tags, _status) = bucket.get_object_tagging("test_file").await?;
            println!("{:?}", tags);

            // Test with random byte array

            let random_bytes: Vec<u8> = (0..3072).map(|_| 33).collect();
            let (_, code) = bucket
                .put_object("random.bin", random_bytes.as_slice())
                .await?;
            assert_eq!(http::StatusCode::OK, code);
            let (data, code) = bucket.get_object("random.bin").await?;
            assert_eq!(code, http::StatusCode::OK);
            assert_eq!(data.len(), 3072);
            assert_eq!(data, random_bytes);
        }

        Ok(())
    }
    fn new_storage(kind: &str) -> Storage {
        let storage = match kind {
            "minio" => Storage {
                name: "minio".into(),
                region: Region::Custom {
                    region: "".into(),
                    endpoint: "http://172.17.0.2:9000".into(),
                },
                credentials: Credentials {
                    access_key: Some("minioadmin".to_owned()),
                    secret_key: Some("minioadmin".to_owned()),
                    security_token: None,
                    session_token: None,
                },
                bucket: "test-bucket".to_string(),
                location_supported: false,
            },
            "aws" => Storage {
                name: "aws".into(),
                region: "eu-central-1".parse().unwrap(),
                credentials: Credentials::from_env_specific(
                    Some("minioadmin"),
                    Some("EU_AWS_SECRET_ACCESS_KEY"),
                    None,
                    None,
                )
                .unwrap(),
                bucket: "rust-s3-test".to_string(),
                location_supported: true,
            },
            "aws_public" => Storage {
                name: "aws-public".into(),
                region: "eu-central-1".parse().unwrap(),
                credentials: Credentials::anonymous().unwrap(),
                bucket: "rust-s3-public".to_string(),
                location_supported: true,
            },
            _ => Storage {
                name: "yandex".into(),
                region: "ru-central1".parse().unwrap(),
                credentials: Credentials::from_profile(Some("yandex")).unwrap(),
                bucket: "soundcloud".to_string(),
                location_supported: false,
            },
        };
        return storage;
    }
}
