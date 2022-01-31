use crate::bar::WrappedBar;
use crate::error::ValidateError;
pub struct Driver;

trait RESTVerbs {
    fn get(url: &str, path: &str, silent: bool);
}

impl Driver {
    async fn get(
        input: &str,
        output: &str,
        expected_sha256: &str,
        bar: &mut WrappedBar,
    ) -> Result<(), ValidateError> {
        let result = match &input[0..4] {
            "ftp:" | "ftp." => {
                crate::ftp::FTPHandler::get(input, output, bar, expected_sha256).await
            }
            "http" => crate::https::HTTPSHandler::get(input, output, bar, expected_sha256).await,
            "ssh:" => crate::ssh::SSHHandler::get(input, output, bar, expected_sha256).await,
            _ => panic!(
                "Cannot extract handler from args: {} {} Exiting.",
                input, output
            ),
        };
        result
    }
    async fn put(input: &str, output: &str, bar: WrappedBar) -> Result<(), ValidateError> {
        let result = match &output[0..4] {
            "ftp:" | "ftp." => crate::ftp::FTPHandler::put(input, output, bar).await,
            "http" => crate::https::HTTPSHandler::put(input, output, bar).await,
            "ssh:" => panic!("Currently not implemented."),
            _ => panic!(
                "Cannot extract handler from args: {} {} Exiting.",
                input, output
            ),
        };
        result
    }

    pub async fn drive(
        input: &str,
        output: &str,
        silent: bool,
        expected_sha256: &str,
    ) -> Result<(), ValidateError> {
        let mut bar = WrappedBar::new(0, input, silent);
        let result = match &input[0..4] {
            "http" | "ftp:" | "ftp." | "ssh:" => {
                Driver::get(input, output, expected_sha256, &mut bar).await
            }
            _ => Driver::put(input, output, bar).await,
        };
        result
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

#[tokio::test]
async fn test_ftp_get_works_when_typical() {
    let result = Driver::get(
        "ftp://ftp.fau.de:21/gnu/MailingListArchives/README",
        "downloaded_ftp_README.md",
        "",
        &mut WrappedBar::new(0, "", true),
    )
    .await;

    assert!(result.is_ok());

    std::fs::remove_file("downloaded_ftp_README.md").unwrap();
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
    async fn test_ssh_get_works_when_typical() {
        let out_file = "_test_ssh_get_works_when_typical";
        just_start("test/ssh/Justfile");
        let result = Driver::get(
            "ssh://user:pass@127.0.0.1:2222/tmp/binfile",
            out_file,
            "aec070645fe53ee3b3763059376134f058cc337247c978add178b6ccdfb0019f",
            &mut WrappedBar::new(0, "", false),
        )
        .await;

        assert!(result.is_ok());

        just_stop("test/ssh/Justfile");
        std::fs::remove_file(out_file).unwrap();
    }
}
