use crate::bar::WrappedBar;
pub struct Driver;

trait RESTVerbs {
    fn get(url: &str, path: &str, silent: bool);
}

impl Driver {
    pub async fn drive(input: &str, output: &str, silent: bool, expected_sha256: &str) -> bool {
        let mut bar = WrappedBar::new(0, input, silent);
        let result = match &input[0..4] {
            "ftp:" | "ftp." => {
                crate::ftp::FTPHandler::get(input, output, &mut bar, expected_sha256).await
            }
            "http" => {
                crate::https::HTTPSHandler::get(input, output, &mut bar, expected_sha256).await
            }
            _ => match &output[0..4] {
                "ftp:" | "ftp." => crate::ftp::FTPHandler::put(input, output, bar).await,
                "http" => crate::https::HTTPSHandler::put(input, output, bar).await,
                _ => panic!(
                    "Cannot extract handler from args: {} {} Exiting.",
                    input, output
                ),
            },
        };
        result
    }
}
