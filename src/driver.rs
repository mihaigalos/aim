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
