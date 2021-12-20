use crate::bar::WrappedBar;
pub struct Driver;

trait RESTVerbs {
    fn get(url: &str, path: &str, silent: bool);
}

impl Driver {
    async fn get(input: &str, output: &str, expected_sha256: &str, bar: &mut WrappedBar) -> bool {
        let result = match &input[0..4] {
            "ftp:" | "ftp." => {
                crate::ftp::FTPHandler::get(input, output, bar, expected_sha256).await
            }
            "http" => crate::https::HTTPSHandler::get(input, output, bar, expected_sha256).await,
            _ => panic!(
                "Cannot extract handler from args: {} {} Exiting.",
                input, output
            ),
        };
        result
    }
    async fn put(input: &str, output: &str, bar: WrappedBar) -> bool {
        let result = match &output[0..4] {
            "ftp:" | "ftp." => crate::ftp::FTPHandler::put(input, output, bar).await,
            "http" => crate::https::HTTPSHandler::put(input, output, bar).await,
            _ => panic!(
                "Cannot extract handler from args: {} {} Exiting.",
                input, output
            ),
        };
        result
    }

    pub async fn drive(input: &str, output: &str, silent: bool, expected_sha256: &str) -> bool {
        let mut bar = WrappedBar::new(0, input, silent);
        let result = match &input[0..4] {
            "http" | "ftp:" | "ftp" => Driver::get(input, output, expected_sha256, &mut bar).await,
            _ => Driver::put(input, output, bar).await,
        };
        result
    }
}

#[tokio::test]
#[should_panic]
async fn test_panics_when_invalid_input() {
    Driver::drive("", "https://foo.bar", true, "").await;
}

#[tokio::test]
#[should_panic]
async fn test_panics_when_invalid_output() {
    Driver::drive("https://foo.bar", "", true, "").await;
}

#[tokio::test]
#[should_panic]
async fn test_get_panics_when_invalid_input() {
    Driver::get("", "", "", &mut WrappedBar::new(0, "", true)).await;
}

#[tokio::test]
#[should_panic]
async fn test_put_panics_when_invalid_input() {
    Driver::put("", "", WrappedBar::new(0, "", true)).await;
}

#[tokio::test]
async fn test_https_get_works_when_typical_input() {
    Driver::get(
        "https://github.com/mihaigalos/aim/blob/main/LICENSE.md",
        "downloaded_https_LICENSE.md",
        "",
        &mut WrappedBar::new(0, "", true),
    )
    .await;
    std::fs::remove_file("downloaded_https_LICENSE.md").unwrap();
}

#[tokio::test]
async fn test_ftp_get_works_when_typical_input() {
    Driver::get(
        "ftp://ftp.fau.de:21/gnu/MailingListArchives/README",
        "downloaded_ftp_README.md",
        "",
        &mut WrappedBar::new(0, "", true),
    )
    .await;
    std::fs::remove_file("downloaded_ftp_README.md").unwrap();
}
