pub struct Driver;

impl Driver {
    pub async fn drive(url: &str, output: &str) {
        match &url[0..4] {
            "ftp:" | "ftp." => crate::ftp::FTPHandler::get(url, output).await,
            "http" => crate::https::HTTPSHandler::get(url, output).await.unwrap(),
            _ => println!("Cannot exctract handler from URL: {} Exiting.", url),
        }
    }
}
