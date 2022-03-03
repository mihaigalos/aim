use std::path::Path;

use crate::untildify::untildify;

fn get_possible_ssh_keys_path(silent: bool) -> String {
    let candidates = vec![
        "id_ed25519",
        "id_rsa",
        "keys/id_ed25519",
        "keys/id_rsa",
        "~/.ssh/id_rsa",
        "~/.ssh/keys/id_ed25519",
    ];
    for candidate in candidates {
        let candidate = untildify(&candidate);
        if Path::new(&candidate).exists() {
            if !silent {
                println!("🔑 Parsed ssh key from: {}", candidate);
            }
            return candidate.to_string();
        }
    }
    return "".to_string();
}

pub fn get_ssh_keys_path(silent: bool) -> Option<String> {
    let mut result = None;
    let path = get_possible_ssh_keys_path(silent);
    if path != "" {
        result = Some(path);
    }
    result
}

#[test]
fn test_get_ssh_keys_path_works_when_typical() {
    let actual = get_ssh_keys_path(true);

    assert!(actual.is_some());
}
