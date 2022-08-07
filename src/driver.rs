use crate::bar::WrappedBar;
use crate::error::ValidateError;
use crate::slicer::Slicer;

use futures::future::BoxFuture;
use melt::decompress;
use std::future::Future;
use std::io;

use futures_util::FutureExt;

use std::collections::HashMap;

type BoxedHandlerFut = Box<dyn Future<Output = Result<(), ValidateError>>>;
// type GetHandler<Return> = Box<dyn Fn(&str, &str, &mut WrappedBar, &str) -> BoxFuture<Return>>;
type GetHandler<'a, Return> =
    Box<dyn Fn(&'a str, &'a str, &'a mut WrappedBar, &'a str) -> BoxFuture<'a, Return>>;
type PutHandler<Return> = Box<dyn Fn(&str, &str, &WrappedBar) -> Return>;

pub fn schema_handlers<'a, Fut>() -> HashMap<&'a str, GetHandler<'a, BoxedHandlerFut>> {
    let mut m = HashMap::<&str, GetHandler<BoxedHandlerFut>>::new();
    m.insert(
        "http",
        Box::new(|a: &_, b: &_, c: &mut _, d: &_| {
            (crate::https::HTTPSHandler::get(a, b, c, d)).boxed()
        }),
    );
    // m.insert(key, Box::new(|input| (Handlers::get(input)).boxed()));

    m
}

// pub fn schema_handlers<Fut>(
//     key: &str,
//     get_handler: impl Fn(&str, &str, &mut WrappedBar, &str) -> Fut + 'static,
//     put_handler: impl Fn(&str, &str, &WrappedBar) -> Fut + 'static,
// ) -> HashMap<&str, (GetHandler<BoxedHandlerFut>, PutHandler<BoxedHandlerFut>)>
// where
//     Fut: Future<Output = Result<(), ValidateError>> + 'static,
// {
//     let mut m = HashMap::new();
//     m.insert(
//         key,
//         (
//             Box::new(move |a: &_, b: &_, c: &mut _, d: &_| {
//                 Box::new(get_handler(a, b, c, d)) as BoxedHandlerFut
//             }) as _,
//             Box::new(move |a: &_, b: &_, c: &_| Box::new(put_handler(a, b, c)) as BoxedHandlerFut)
//                 as _,
//         ),
//     );
//     m
// }

// pub fn default_schema_handlers<'a>(
// ) -> HashMap<&'a str, (GetHandler<BoxedHandlerFut>, PutHandler<BoxedHandlerFut>)> {
//     let result = HashMap::new();
//     schema_handlers(
//         "http",
//         crate::https::HTTPSHandler::get,
//         crate::https::HTTPSHandler::put,
//     );

//     result
// }

pub struct Driver;
impl Driver {
    async fn get(
        input: &str,
        output: &str,
        expected_sha256: &str,
        bar: &mut WrappedBar,
    ) -> io::Result<()> {
        let (output, is_decompress_requested) = match output {
            "." => (Slicer::target_with_extension(input), false),
            "+" => (Slicer::target_with_extension(input), true),
            _ => (output, false),
        };

        let result = match &input[0..4] {
            "ftp:" | "ftp." => {
                crate::ftp::FTPHandler::get(input, output, bar, expected_sha256).await?
            }
            "http" => crate::https::HTTPSHandler::get(input, output, bar, expected_sha256).await?,
            "sftp" => crate::sftp::SFTPHandler::get(input, output, bar, expected_sha256).await?,
            "ssh:" => crate::ssh::SSHHandler::get(input, output, bar, expected_sha256).await?,
            "s3:/" => crate::s3::S3::get(input, output, bar, expected_sha256).await?,
            _ => panic!(
                "Cannot extract handler from args: {} {} Exiting.",
                input, output
            ),
        };

        if is_decompress_requested {
            decompress(std::path::Path::new(output)).unwrap();
            std::fs::remove_file(output)?;
        }
        Ok(result)
    }

    async fn put(input: &str, output: &str, bar: WrappedBar) -> io::Result<()> {
        let result = match &output[0..4] {
            "ftp:" | "ftp." => crate::ftp::FTPHandler::put(input, output, bar).await?,
            "http" => crate::https::HTTPSHandler::put(input, output, bar).await?,
            "sftp" => crate::sftp::SFTPHandler::put(input, output, bar).await?,
            "ssh:" => crate::ssh::SSHHandler::put(input, output, bar).await?,
            "s3:/" => crate::s3::S3::put(input, output, bar).await?,
            _ => panic!(
                "Cannot extract handler from args: {} {} Exiting.",
                input, output
            ),
        };
        Ok(result)
    }

    pub async fn drive(
        input: &str,
        output: &str,
        silent: bool,
        expected_sha256: &str,
    ) -> io::Result<()> {
        let mut bar = WrappedBar::new(0, input, silent);

        if input.contains("http:")
            || input.contains("https:")
            || input.contains("ftp:")
            || input.contains("sftp:")
            || input.contains("ssh:")
            || input.contains("s3:")
        {
            return Ok(Driver::get(input, output, expected_sha256, &mut bar).await?);
        } else {
            return match output {
                "stdout" => {
                    Ok(crate::http_serve_folder::WarpyWrapper::run(input.to_string()).await)
                }
                _ => Ok(Driver::put(input, output, bar).await?),
            };
        }
    }
}

#[tokio::test]
#[should_panic]
async fn test_panics_when_invalid_output() {
    let _ = Driver::drive("", "https://foo.bar", true, "").await;
}

#[tokio::test]
#[should_panic]
async fn test_panics_when_invalid_input() {
    let _ = Driver::drive("https://foo.bar", "", true, "").await;
}

#[tokio::test]
#[should_panic]
async fn test_get_panics_when_invalid_input() {
    let _ = Driver::get("invalid", "", "", &mut WrappedBar::new(0, "", true)).await;
}

#[tokio::test]
#[should_panic]
async fn test_put_panics_when_invalid_input() {
    let _ = Driver::put("", "invalid", WrappedBar::new(0, "", true)).await;
}

#[tokio::test]
async fn test_driver_works_when_typical() {
    let result = Driver::drive(
        "https://github.com/mihaigalos/aim/blob/main/LICENSE.md",
        "downloaded_driver_https_LICENSE.md",
        true,
        "",
    )
    .await;

    assert!(result.is_ok());

    std::fs::remove_file("downloaded_driver_https_LICENSE.md").unwrap();
}

#[tokio::test]
async fn test_https_get_works_when_typical() {
    let result = Driver::get(
        "https://github.com/mihaigalos/aim/blob/main/LICENSE.md",
        "downloaded_https_LICENSE.md",
        "",
        &mut WrappedBar::new(0, "", true),
    )
    .await;

    assert!(result.is_ok());

    std::fs::remove_file("downloaded_https_LICENSE.md").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn just_start(justfile: &str) {
        use std::env;
        use std::io::Write;
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
        use std::io::Write;
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
    async fn test_driver_https_put_works_when_typical() {
        just_start("test/https/Justfile");

        let result = Driver::drive(
            "test/https/binary_file.tar.gz",
            "http://127.0.0.1:8081/_test_aim_driver_https_put_binary_file",
            true,
            "",
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/https/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_driver_https_get_decompress_works_when_typical() {
        let out_file = "test_driver_https_get_extract_works_when_typical.tar.gz";
        just_start("test/https/Justfile");

        let _ = Driver::drive(
            "test/https/compressed_file.tar.gz",
            &("http://127.0.0.1:8081/".to_string() + out_file),
            true,
            "",
        )
        .await;
        let result = Driver::drive(
            &("http://127.0.0.1:8081/".to_string() + out_file),
            "+",
            true,
            "",
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/https/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_https_put_works_when_typical() {
        just_start("test/https/Justfile");

        let result = Driver::put(
            "test/https/binary_file.tar.gz",
            "http://user:pass@127.0.0.1:8081/_test_aim_put_binary_file",
            WrappedBar::new(0, "", true),
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/https/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_ftp_put_works_when_typical() {
        just_start("test/ftp/Justfile");
        let result = Driver::put(
            "test/ftp/binary_file.tar.gz",
            "ftp://127.0.0.1:21/_test_aim_put_binary_file",
            WrappedBar::new(0, "", true),
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/ftp/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_ftp_put_works_when_subfolder() {
        just_start("test/ftp/Justfile");
        let result = Driver::put(
            "test/ftp/binary_file.tar.gz",
            "ftp://127.0.0.1:21/subfolder/test_ftp_put_works_when_subfolder",
            WrappedBar::new(0, "", true),
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/ftp/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_ftp_get_works_same_filename() {
        just_start("test/ftp/Justfile");
        let out_file = ".";
        let expected_hash = "cc7e91ef8d68d0c0e06857e0713e490d4cead4164f99c9dc1a59c3e93e217a6d";
        let _ = Driver::put(
            "test/ftp/binary_file.tar.gz",
            "ftp://127.0.0.1:21/test_ftp_get_works_same_filename",
            WrappedBar::new(0, "", true),
        )
        .await;
        let result = Driver::get(
            "ftp://127.0.0.1:21/test_ftp_get_works_same_filename",
            out_file,
            expected_hash,
            &mut WrappedBar::new(0, "", true),
        )
        .await;
        std::fs::remove_file("test_ftp_get_works_same_filename").unwrap();
        assert!(result.is_ok());
        just_stop("test/ftp/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_ftp_get_resume_works() {
        just_start("test/ftp/Justfile");
        let expected_hash = "cc7e91ef8d68d0c0e06857e0713e490d4cead4164f99c9dc1a59c3e93e217a6d";
        let out_file = "test_get_ftp_resume_works";

        let _ = Driver::put(
            "test/ftp/binary_file.tar.gz",
            "ftp://127.0.0.1:21/binary_file.tar.gz",
            WrappedBar::new(0, "", true),
        )
        .await;
        std::fs::copy("test/ftp/binary_file.tar.gz.part1", out_file).unwrap();
        let result = Driver::get(
            "ftp://127.0.0.1:21/binary_file.tar.gz",
            out_file,
            expected_hash,
            &mut WrappedBar::new(0, "", true),
        )
        .await;

        println!("out file: {}", out_file);
        assert!(result.is_ok());
        std::fs::remove_file(out_file).unwrap();
        just_stop("test/ftp/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_ssh_get_works_when_typical() {
        let out_file = "_test_ssh_get_works_when_typical";
        just_start_with_keys("test/ssh/Justfile");
        let result = Driver::get(
            "ssh://user@127.0.0.1:2223/tmp/foobar_keys",
            out_file,
            "364f419c559bd3eb24434b97353cfaa4792cc70c9151f9cd8274bbe16b42a29a",
            &mut WrappedBar::new(0, "", false),
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/ssh/Justfile");
        std::fs::remove_file(out_file).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_ssh_put_works_when_typical() {
        just_start_with_keys("test/ssh/Justfile");

        let result = Driver::put(
            "test/ssh/binary_file.tar.gz",
            "ssh://user@127.0.0.1:2223/tmp/_test_ssh_put_works_when_typical",
            WrappedBar::new(0, "", false),
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/ssh/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_sftp_get_works_when_typical() {
        let out_file = "_test_sftp_get_works_when_typical";
        just_start_with_keys("test/ssh/Justfile");
        let result = Driver::get(
            "sftp://user@127.0.0.1:2223/tmp/foobar_keys",
            out_file,
            "364f419c559bd3eb24434b97353cfaa4792cc70c9151f9cd8274bbe16b42a29a",
            &mut WrappedBar::new(0, "", false),
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/ssh/Justfile");
        std::fs::remove_file(out_file).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_sftp_put_works_when_typical() {
        just_start_with_keys("test/ssh/Justfile");

        let result = Driver::put(
            "test/ssh/binary_file.tar.gz",
            "sftp://user@127.0.0.1:2223/tmp/_test_sftp_put_works_when_typical",
            WrappedBar::new(0, "", false),
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/ssh/Justfile");
    }

    #[tokio::test]
    #[serial]
    async fn test_s3_get_works_when_typical() {
        let out_file = "test_s3_get_works_when_typical";
        just_start("test/s3/Justfile");

        let result = Driver::drive(
            "s3://minioadmin:minioadmin@localhost:9000/test-bucket/binary_file.tar.gz.part1",
            out_file,
            true,
            "",
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/s3/Justfile");
        std::fs::remove_file(out_file).unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_s3_put_works_when_typical() {
        let in_file = "test/s3/test.file";
        just_start("test/s3/Justfile");

        let result = Driver::drive(
            in_file,
            "s3://minioadmin:minioadmin@localhost:9000/test-bucket/test.file",
            true,
            "",
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/s3/Justfile");
    }
}

#[tokio::test]
async fn test_http_serve_folder_works_when_typical() {
    tokio::spawn(async {
        let _ = crate::http_serve_folder::WarpyWrapper::run(".".to_string()).await;
    });

    use tokio::time::*;
    sleep(Duration::from_millis(2000)).await;
    let result = Driver::get(
        "http://127.0.0.1:8080/test/http_serve_folder/test.file",
        "downloaded_test_http_serve_folder_works_when_typical",
        "",
        &mut WrappedBar::new(0, "", true),
    )
    .await;

    assert!(result.is_ok());

    std::fs::remove_file("downloaded_test_http_serve_folder_works_when_typical").unwrap();
}
