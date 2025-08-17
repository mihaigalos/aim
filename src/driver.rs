use skim_navi::Navi;

use crate::bar::WrappedBar;
use crate::error::ValidateError;
use crate::slicer::Slicer;

use futures::future::BoxFuture;
use melt::decompress;
use std::io;
use std::io::Error;

pub struct Options {
    pub silent: bool,
    pub interactive: bool,
    pub expected_sha256: String,
}

use url_parse::core::{scheme_separator::SchemeSeparator, Parser};
use url_parse::utils::Utils;

use futures_util::FutureExt;

use std::collections::HashMap;

type GetPutResult = Result<(), ValidateError>;
type ListResult = Result<Vec<String>, Error>;
type GetHandler<'a, Return> =
    Box<dyn Fn(&'a str, &'a str, &'a mut WrappedBar, &'a str) -> BoxFuture<'a, Return>>;
type PutHandler<'a, Return> = Box<dyn Fn(&'a str, &'a str, WrappedBar) -> BoxFuture<'a, Return>>;
type ListHandler<'a, Return> = Box<dyn Fn(String) -> BoxFuture<'a, Return>>;

struct Handlers<'a> {
    get_handler: GetHandler<'a, GetPutResult>,
    put_handler: PutHandler<'a, GetPutResult>,
    list_handler: ListHandler<'a, ListResult>,
}

impl<'a> Handlers<'a> {
    pub fn new(
        get_handler: GetHandler<'a, GetPutResult>,
        put_handler: PutHandler<'a, GetPutResult>,
        list_handler: ListHandler<'a, ListResult>,
    ) -> Self {
        Self {
            get_handler,
            put_handler,
            list_handler,
        }
    }
}

fn schema_handlers<'a>() -> HashMap<&'a str, Handlers<'a>> {
    let mut m = HashMap::<&str, Handlers>::new();

    m.insert(
        "ftp",
        Handlers::new(
            Box::new(move |a: &_, b: &_, c: &mut _, d: &_| {
                crate::ftp::FTPHandler::get(a, b, c, d).boxed()
            }),
            Box::new(move |a: &_, b: &_, c: _| crate::ftp::FTPHandler::put(a, b, c).boxed()),
            Box::new(move |a: _| crate::ftp::FTPHandler::get_links(a).boxed()),
        ),
    );
    m.insert(
        "http",
        Handlers::new(
            Box::new(move |a: &_, b: &_, c: &mut _, d: &_| {
                crate::https::HTTPSHandler::get(a, b, c, d).boxed()
            }),
            Box::new(move |a: &_, b: &_, c: _| crate::https::HTTPSHandler::put(a, b, c).boxed()),
            Box::new(move |a: _| crate::https::HTTPSHandler::get_links(a).boxed()),
        ),
    );
    m.insert(
        "https",
        Handlers::new(
            Box::new(move |a: &_, b: &_, c: &mut _, d: &_| {
                crate::https::HTTPSHandler::get(a, b, c, d).boxed()
            }),
            Box::new(move |a: &_, b: &_, c: _| crate::https::HTTPSHandler::put(a, b, c).boxed()),
            Box::new(move |a: _| crate::https::HTTPSHandler::get_links(a).boxed()),
        ),
    );
    m.insert(
        "sftp",
        Handlers::new(
            Box::new(move |a: &_, b: &_, c: &mut _, d: &_| {
                crate::sftp::SFTPHandler::get(a, b, c, d).boxed()
            }),
            Box::new(move |a: &_, b: &_, c: _| crate::sftp::SFTPHandler::put(a, b, c).boxed()),
            Box::new(move |a: _| crate::sftp::SFTPHandler::get_links(a).boxed()),
        ),
    );
    m.insert(
        "ssh",
        Handlers::new(
            Box::new(move |a: &_, b: &_, c: &mut _, d: &_| {
                crate::ssh::SSHHandler::get(a, b, c, d).boxed()
            }),
            Box::new(move |a: &_, b: &_, c: _| crate::ssh::SSHHandler::put(a, b, c).boxed()),
            Box::new(move |a: _| crate::ssh::SSHHandler::get_links(a).boxed()),
        ),
    );
    m.insert(
        "s3",
        Handlers::new(
            Box::new(move |a: &_, b: &_, c: &mut _, d: &_| crate::s3::S3::get(a, b, c, d).boxed()),
            Box::new(move |a: &_, b: &_, c: _| crate::s3::S3::put(a, b, c).boxed()),
            Box::new(move |a: _| crate::s3::S3::get_links(a).boxed()),
        ),
    );
    m
}

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

        let scheme = Driver::extract_scheme_or_panic(input);
        let schema_handlers = schema_handlers();
        (schema_handlers[scheme.0].get_handler)(input, output, bar, expected_sha256).await?;

        if is_decompress_requested {
            decompress(std::path::Path::new(output)).unwrap();
            std::fs::remove_file(output)?;
        }
        Ok(())
    }

    async fn put(input: &str, output: &str, bar: WrappedBar) -> io::Result<()> {
        let scheme = Driver::extract_scheme_or_panic(output);
        let schema_handlers = schema_handlers();
        (schema_handlers[scheme.0].put_handler)(input, output, bar).await?;
        Ok(())
    }

    pub async fn dispatch(input: &str, output: &str, options: &Options) -> io::Result<()> {
        let input = &Self::navigate(input, options).await;
        Driver::drive(input, output, options.silent, &options.expected_sha256).await
    }

    async fn drive(
        input: &str,
        output: &str,
        silent: bool,
        expected_sha256: &str,
    ) -> io::Result<()> {
        let mut bar = WrappedBar::new(0, input, silent);
        let scheme = Parser::new(None).scheme(input);
        if scheme.is_some() {
            Driver::get(input, output, expected_sha256, &mut bar).await?;
            Ok(())
        } else {
            match output {
                "stdout" => {
                    crate::http_serve_folder::WarpyWrapper::run(input.to_string()).await?;
                    Ok(())
                }
                _ => Ok(Driver::put(input, output, bar).await?),
            }
        }
    }

    fn extract_scheme(address: &str) -> (&str, SchemeSeparator) {
        Parser::new(None)
            .scheme(address)
            .unwrap_or(("", SchemeSeparator::ColonSlashSlash))
    }

    fn extract_scheme_or_panic(address: &str) -> (&str, SchemeSeparator) {
        let scheme = Parser::new(None).scheme(address);
        if scheme.is_none() {
            panic!("Cannot extract handler from arg: {address} Exiting.");
        }
        scheme.unwrap()
    }

    #[cfg(not(tarpaulin_include))]
    async fn navigate(input: &str, options: &Options) -> String {
        let scheme = Driver::extract_scheme(input);
        let schema_handlers = schema_handlers();
        let path = match options.interactive {
            false => "".to_string(),
            true => Navi::run(input, &schema_handlers[scheme.0].list_handler)
                .await
                .unwrap_or("".to_string() + "/"),
        };

        if !path.is_empty() {
            let parser = Parser::new(None);
            return Utils::canonicalize(&parser, input, &path);
        }
        input.to_string()
    }
}

#[test]
fn test_extract_scheme_works_when_typical() {
    let expected = "";
    let (result, _) = Driver::extract_scheme(".");
    assert_eq!(result, expected);
}

#[test]
fn test_extract_scheme_or_panic_works_when_typical() {
    let expected_scheme = "https";
    let expected_separator = SchemeSeparator::ColonSlashSlash;
    let result = Driver::extract_scheme_or_panic("https://foo.bar");
    assert_eq!(result.0, expected_scheme);
    assert_eq!(result.1, expected_separator);
}

#[test]
#[should_panic]
fn test_extract_scheme_or_panic_panics_when_no_scheme() {
    Driver::extract_scheme_or_panic("foo.bar");
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
async fn test_dispatch_works_when_typical() {
    let result = Driver::dispatch(
        "https://github.com/mihaigalos/aim/blob/main/LICENSE.md",
        "downloaded_driver_https_dispatch_LICENSE.md",
        &Options {
            silent: true,
            interactive: false,
            expected_sha256: "".to_string(),
        },
    )
    .await;

    assert!(result.is_ok());

    std::fs::remove_file("downloaded_driver_https_dispatch_LICENSE.md").unwrap();
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

        println!("out file: {out_file}");
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

#[tokio::test]
async fn test_hashed_handlers_created_correctly_when_typical() {
    let schema_handlers = schema_handlers();

    for item in ["http", "https", "ftp", "sftp", "ssh", "s3"] {
        assert!(schema_handlers.contains_key(item));
    }
}

#[tokio::test]
async fn test_hashed_handlers_https_list_works_when_typical() {
    let input = "https://github.com/XAMPPRocky/tokei/releases/";
    let scheme = Driver::extract_scheme_or_panic(input);
    let schema_handlers = schema_handlers();

    let result = (schema_handlers[scheme.0].list_handler)(input.to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_hashed_handlers_http_list_works_when_typical() {
    let input = "http://github.com/XAMPPRocky/tokei/releases/";
    let scheme = Driver::extract_scheme_or_panic(input);
    let schema_handlers = schema_handlers();

    let result = (schema_handlers[scheme.0].list_handler)(input.to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[should_panic]
async fn test_hashed_handlers_ftp_list_works_when_typical() {
    let input = "ftp://unimplemented";
    let scheme = Driver::extract_scheme_or_panic(input);
    let schema_handlers = schema_handlers();

    let result = (schema_handlers[scheme.0].list_handler)(input.to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[should_panic]
async fn test_hashed_handlers_sftp_list_works_when_typical() {
    let input = "sftp://unimplemented";
    let scheme = Driver::extract_scheme_or_panic(input);
    let schema_handlers = schema_handlers();

    let result = (schema_handlers[scheme.0].list_handler)(input.to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[should_panic]
async fn test_hashed_handlers_ssh_list_works_when_typical() {
    let input = "ssh://unimplemented";
    let scheme = Driver::extract_scheme_or_panic(input);
    let schema_handlers = schema_handlers();

    let result = (schema_handlers[scheme.0].list_handler)(input.to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[should_panic]
async fn test_hashed_handlers_s3_list_works_when_typical() {
    let input = "s3://unimplemented";
    let scheme = Driver::extract_scheme_or_panic(input);
    let schema_handlers = schema_handlers();

    let result = (schema_handlers[scheme.0].list_handler)(input.to_string()).await;
    assert!(result.is_ok());
}
