use netrc::Netrc;
use std::env;
use std::io::BufReader;
use std::path::Path;

fn get_possible_netrc_path() -> String {
    let current_directory = env::var("PWD").unwrap_or("".to_string());
    let candidates = vec![current_directory + "/.netrc", "~/.netrc".to_string()];
    for cantidate in candidates {
        if Path::new(&cantidate).exists() {
            return cantidate;
        }
    }
    return "".to_string();
}

pub fn netrc() -> Option<netrc::Netrc> {
    let mut result = None;
    let path = get_possible_netrc_path();
    if path != "" {
        let in_ = BufReader::new(path.as_bytes());
        result = Some(Netrc::parse(in_).unwrap());
    }
    result
}
