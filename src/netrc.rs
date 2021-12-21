use netrc::Netrc;
use std::env;
use std::io::BufReader;
use std::path::Path;

fn get_possible_netrc_path() -> String {
    let current_directory = env::var("PWD").unwrap_or("".to_string());
    let candidates = vec![
        current_directory.clone() + "/.netrc",
        current_directory.clone() + "/.netrc.test",
        current_directory.clone() + "/.netrc.test.ftp",
        current_directory.clone() + "/.netrc.test.https",
        current_directory.clone() + "/.netrc.test.unit",
        "~/.netrc".to_string(),
    ];
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
        let file = std::fs::File::open(path).unwrap();
        let parsed = Netrc::parse(BufReader::new(file));
        result = Some(parsed.unwrap());
    }
    result
}

#[test]
fn test_netrc_with_file_works_when_typical() {
    use std::io::Write;
    let mut file = std::fs::File::create(".netrc.test.unit").unwrap();
    file.write_all(b"machine mydomain.com login myuser password mypass port 1234")
        .unwrap();

    let netrc = netrc();

    assert!(netrc.is_some());
    std::fs::remove_file(".netrc.test.unit").unwrap();
}
