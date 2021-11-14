pub struct Driver;

impl Driver {
    pub async fn drive(input: &str, output: &str, silent: bool) {
        match &input[0..4] {
            "ftp:" | "ftp." => crate::ftp::FTPHandler::get(input, output, silent).await,
            "http" => crate::https::HTTPSHandler::get(input, output, silent)
                .await
                .unwrap(),
            _ => println!("Cannot extract handler from input: {} Exiting.", input),
        }
    }
}
