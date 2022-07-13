use std::path::Path;

use untildify::untildify;

pub fn get_possible_ssh_keys_path(silent: bool) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    let candidates = vec![
        "id_ed25519",
        "id_rsa",
        "keys/id_ed25519",
        "keys/id_rsa",
        "test/ssh/keys/id_ed25519",
        "test/ssh/keys/id_rsa",
        "~/.ssh/id_rsa",
        "~/.ssh/id_ed25519",
    ];
    for candidate in candidates {
        let candidate = untildify(&candidate);
        if Path::new(&candidate).exists() {
            if !silent {
                println!("🔑 Parsed ssh key from: {}", candidate);
            }
            result.push(candidate.to_string());
        }
    }
    result
}

#[test]
fn test_get_possible_ssh_keys_path_when_typical() {
    let actual = get_possible_ssh_keys_path(false);

    assert!(actual.len() > 0);
}
