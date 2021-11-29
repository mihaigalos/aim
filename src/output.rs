use std::fs::File;
use std::io::{Read, Write};

fn get_output_file(path: &str, silent: bool) -> (Option<std::fs::File>, u64) {
    let mut transfered: u64 = 0;
    let mut file = None;
    if path != "stdout" {
        if std::path::Path::new(path).exists() {
            if !silent {
                println!("File exists. Resuming.");
            }
            file = Some(
                std::fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(path)
                    .unwrap(),
            );

            let file_size = std::fs::metadata(path).unwrap().len();
            transfered = file_size;
        } else {
            if !silent {
                println!("Writing to new file.");
            }
            file = Some(
                File::create(path)
                    .or(Err(format!("Failed to create file '{}'", path)))
                    .unwrap(),
            );
        }
    }
    (file, transfered)
}

fn get_input_file(path: &str, silent: bool) -> (Option<std::fs::File>, u64) {
    let mut transfered: u64 = 0;
    let mut file = None;
    if path != "stdin" {
        if std::path::Path::new(path).exists() {
            file = Some(std::fs::OpenOptions::new().read(true).open(path).unwrap());

            let file_size = std::fs::metadata(path).unwrap().len();
            transfered = file_size;
        } else {
            if !silent {
                println!("Writing to new file.");
            }
            file = Some(
                File::create(path)
                    .or(Err(format!("Failed to create file '{}'", path)))
                    .unwrap(),
            );
        }
    }
    (file, transfered)
}

pub fn get_output(path: &str, silent: bool) -> (Box<dyn Write>, u64) {
    let (file, transfered) = get_output_file(path, silent);
    let output: Box<dyn Write> = Box::new(std::io::BufWriter::new(match path {
        "stdout" => Box::new(std::io::stdout()) as Box<dyn Write>,
        _ => Box::new(file.unwrap()) as Box<dyn Write>,
    }));

    (output, transfered)
}

pub fn get_input(path: &str, silent: bool) -> (Box<dyn Read>, u64) {
    let (file, transfered) = get_input_file(path, silent);
    let output: Box<dyn Read> = Box::new(std::io::BufReader::new(match path {
        "stdin" => Box::new(std::io::stdin()) as Box<dyn Read>,
        _ => Box::new(file.unwrap()) as Box<dyn Read>,
    }));

    (output, transfered)
}
