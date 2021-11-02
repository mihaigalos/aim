// ftp://ftp.fau.de/archlinux/pool/packages/ffmpeg-2:4.4-6-x86_64.pkg.tar.zst
// https://ftp.fau.de/archlinux/pool/packages/ffmpeg-2:4.4-6-x86_64.pkg.tar.zst

// https://ftp.fau.de/archlinux/lastsync

use url::Url;
use failure::format_err;

use async_ftp::FtpStream;
fn parse_ftp_address(address: &str) -> (String, String, String, Vec<String>, String) {
    let url = Url::parse(address).unwrap();
    let ftp_server = format!(
        "{}:{}",
        url .host_str()
            .ok_or_else(|| format_err!("failed to parse hostname from url: {}", url))
            .unwrap(),
        url .port_or_known_default()
            .ok_or_else(|| format_err!("failed to parse port from url: {}", url))
            .unwrap(),
    );
    let username = if url.username().is_empty() {
        "anonymous".to_string()
    } else {
        url.username().to_string()
    };
    let password = url.password().unwrap_or("anonymous").to_string();

    let mut path_segments: Vec<String> = url
        .path_segments()
        .ok_or_else(|| format_err!("failed to get url path segments: {}", url))
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let file = path_segments
        .pop()
        .ok_or_else(|| format_err!("got empty path segments from url: {}", url))
        .unwrap();

    (ftp_server, username, password, path_segments, file.to_string())
}
pub async fn get() -> String {

    let (ftp_server, ref username, ref password, path_segments, ref file) = parse_ftp_address("ftp://ftp.fau.de:21/gnu/ProgramIndex");

    let mut ftp_stream = FtpStream::connect(ftp_server).await.unwrap();
    let _ = ftp_stream.login(username, password).await.unwrap();
    for path in &path_segments {
        ftp_stream.cwd(&path).await.unwrap();
    }

    let remote_file = ftp_stream.simple_retr(file).await.unwrap();
    let contents = std::str::from_utf8(&remote_file.into_inner()).unwrap().to_string();
    println!("Read file with contents\n{}\n", contents);

    let _ = ftp_stream.quit();
    contents
}

#[tokio::test]
async fn get_ftp_works() {
    let contents = get().await;
    let first_line = contents.split(' ').collect::<Vec<&str>>();
    assert!(first_line[0].starts_with("Here"));
}