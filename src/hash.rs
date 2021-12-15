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
        drop(file);

        format!("{:x}", computed_hash)
    }
}

#[test]
fn test_sha256sum_api() {
    let expected = "800dbea8f23421c6306df712af6f416a3f570ecf5652b45fd6d409019fe6d4fe";

    let actual = HashChecker::sha256sum("LICENSE.md");

    assert_eq!(actual, expected);
}

#[test]
fn test_check_api() {
    let silent = false;
    let expected = "800dbea8f23421c6306df712af6f416a3f570ecf5652b45fd6d409019fe6d4fe";

    let is_match = HashChecker::check("LICENSE.md", expected, silent);

    assert!(is_match);
}
