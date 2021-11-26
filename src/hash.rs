use std::process::Command;
use std::str;

pub fn sha256sum( filename: &str) -> String{
    let computed_hash : String = str::from_utf8(&Command::new("sh")
            .arg("-c")
            .arg(&("sha256sum ".to_string() + filename + "| cut -d' ' -f1 | tr -d '\n'"))
            .output()
            .expect("failed to execute process")
            .stdout).unwrap().to_string();
   computed_hash 
}