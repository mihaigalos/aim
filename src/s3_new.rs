use bytes::BytesMut;
use futures::TryStreamExt;
use rusoto_core::credential::{AwsCredentials, DefaultCredentialsProvider, StaticProvider};
use rusoto_core::request::{HttpClient, HttpConfig, TlsError};
use rusoto_core::Region;
use rusoto_credential::ChainProvider;
use rusoto_s3::{GetObjectRequest, ListObjectsV2Request, S3Client, S3};

pub async fn list_objs(client: S3Client, bucket: String, prefix: String) {
    let list_obj_req = ListObjectsV2Request {
        bucket,
        prefix: Some(prefix),
        ..ListObjectsV2Request::default()
    };

    println!("Request: {:?}", list_obj_req);

    let objects = client
        .list_objects_v2(list_obj_req)
        .await
        .unwrap()
        .contents
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();

    println!("Result: {:?}", objects);
}

pub async fn bucket_obj_bytes(client: S3Client, bucket: String, _prefix: String, object: String) {
    let get_req = GetObjectRequest {
        bucket,
        key: object,
        ..Default::default()
    };

    let result = client
        .get_object(get_req)
        .await
        .expect("Couldn't GET object");

    let stream = result.body.unwrap();
    let body = stream
        .map_ok(|b| BytesMut::from(&b[..]))
        .try_concat()
        .await
        .unwrap();

    assert!(body.len() > 0);
    dbg!(body);
}

pub fn new_s3client_with_credentials(
    region: Region,
    access_key: String,
    secret_key: String,
) -> Result<S3Client, TlsError> {
    Ok(S3Client::new_with(
        HttpClient::new()?,
        StaticProvider::new_minimal(access_key, secret_key),
        region,
    ))
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

    fn just_start_with_keys(justfile: &str) {
        use std::env;
        use std::io::{self, Write};
        use std::process::Command;
        let output = Command::new("just")
            .args([
                "--justfile",
                justfile,
                "_start_with_keys",
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
    async fn test_s3_rusoto_when_typical() {
        // let mut http_config_with_bigger_buffer = HttpConfig::new();
        // http_config_with_bigger_buffer.read_buf_size(1024 * 1024 * 2);
        // let http_provider = HttpClient::new_with_config(http_config_with_bigger_buffer).unwrap();

        // let region = Region::Custom {
        //     name: "us-east-1".to_owned(),
        //     endpoint: "http://localhost:9000".to_owned(),
        // };
        // let credentials_provider = ChainProvider::new();

        // let s3 = S3Client::new_with(http_provider, credentials_provider, region);
        just_start("test/s3/Justfile");
        let test_client = new_s3client_with_credentials(
            Region::Custom {
                name: "".to_owned(),
                endpoint: "http://localhost:9000".to_owned(),
            },
            "minioadmin".to_owned(),
            "minioadmin".to_owned(),
        )
        .unwrap();

        // let s3 = S3Client::new(region);
        let bucket = "test-bucket".to_string();
        let prefix = "".to_string();
        let obj_test = "binary_file.tar.gz.part1".to_string();

        //let _objects = list_objs(s3, bucket, prefix).await;
        let _bytes = bucket_obj_bytes(test_client, bucket, prefix, obj_test).await;
        just_stop("test/s3/Justfile");
    }
}
