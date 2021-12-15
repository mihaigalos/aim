use std::str;

use sha2::{Digest, Sha256};
use std::{fs, io};

pub struct HashChecker;
impl HashChecker {
    pub fn check(filename: &str, expected_hash: &str, silent: bool) -> bool {
        let actual_hash = HashChecker::sha256sum(filename);
        let mut result = true;
        if filename != "stdout" && (expected_hash != "") {
            if actual_hash != expected_hash {
                result = false;
            }
        }
        if !silent && expected_hash != "" {
            if result {
                println!("✅ Checksum OK.");
            } else {
                println!(
                    "❌ Checksum verification failed for {}:\n  expected: {}\n  got:      {}",
                    filename, expected_hash, actual_hash
                );
            }
        }
        result
    }

    fn sha256sum(filename: &str) -> String {
        let mut hasher = Sha256::new();
        let mut file = fs::File::open(filename).unwrap();

        io::copy(&mut file, &mut hasher).unwrap();
        let computed_hash = hasher.finalize();

        format!("{:x}", computed_hash)
    }
}

#[test]
fn test_sha256sum_api() {
    let expected = "0352bbf93e78e3f11f25e6f0271a002f13c64761b8b17985cde0e33651b951df";

    let actual =
        HashChecker::sha256sum("test/incomplete_wpa_supplicant-2XX2.9-8-x86_64.pkg.tar.zst");

    assert_eq!(actual, expected);
}

#[test]
fn test_check_api() {
    let silent = true;
    let expected = "0352bbf93e78e3f11f25e6f0271a002f13c64761b8b17985cde0e33651b951df";

    let is_match = HashChecker::check(
        "test/incomplete_wpa_supplicant-2XX2.9-8-x86_64.pkg.tar.zst",
        expected,
        silent,
    );

    assert!(is_match);
}
