use crate::bar::WrappedBar;
pub struct Driver;

trait RESTVerbs {
    fn get(url: &str, path: &str, silent: bool);
}

impl Driver {
    pub async fn drive(input: &str, output: &str, silent: bool) {
        let bar = WrappedBar::new(0, input, silent);
        match &input[0..4] {
            "ftp:" | "ftp." => crate::ftp::FTPHandler::get(input, output, &bar).await,
            "http" => crate::https::HTTPSHandler::get(input, output, &bar).await,
            _ => match &output[0..4] {
                "ftp:" | "ftp." => crate::ftp::FTPHandler::put(input, output, &bar).await,
                "http" => crate::https::HTTPSHandler::put(input, output, bar).await,
                _ => println!(
                    "Cannot extract handler from args: {} {} Exiting.",
                    input, output
                ),
            },
        }
    }
}
