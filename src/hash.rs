use std::process::Command;
use std::str;

pub struct HashChecker;
impl HashChecker{
    pub fn check(filename: &str, expected_hash: &str, verbose: bool) -> bool{
        let actual_hash = HashChecker::sha256sum(filename);
        let result = actual_hash == expected_hash;
        if verbose {
            if result {
                println!("✅ Checksum OK.");
            } else {
                println!("❌ Checksum verification failed for {}:\n  expected:{}\n  got:{}",filename, expected_hash, actual_hash);
            }
        }
        result
    }

    pub fn sha256sum( filename: &str) -> String{
        let computed_hash : String = str::from_utf8(&Command::new("sh")
                .arg("-c")
                .arg(&("sha256sum ".to_string() + filename + "| cut -d' ' -f1 | tr -d '\n'"))
                .output()
                .expect("failed to execute process")
                .stdout).unwrap().to_string();
    computed_hash 
    }

}
