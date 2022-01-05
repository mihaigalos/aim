use netrc::Netrc;
use std::io::BufReader;
use std::path::Path;

fn get_possible_netrc_path() -> String {
    let candidates = vec![
        ".netrc",
        ".netrc.test",
        ".netrc.test_https",
        ".netrc.test_unit",
        "~/.netrc",
    ];
    for candidate in candidates {
        if Path::new(&candidate).exists() {
            println!("🔑 Parsed .netrc from: {}", candidate);
            return candidate.to_string();
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
    let mut file = std::fs::File::create(".netrc.test_unit").unwrap();
    file.write_all(b"machine mydomain.com login myuser password mypass port 1234")
        .unwrap();

    let netrc = netrc();

    assert!(netrc.is_some());
    std::fs::remove_file(".netrc.test_unit").unwrap();
}
