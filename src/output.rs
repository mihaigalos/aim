use std::fs::File;
use std::io::Write;

fn get_file(path: &str) -> (Option<std::fs::File>, u64) {
    let mut downloaded: u64 = 0;
    let mut file = None;
    if path.len() > 0 {
        if std::path::Path::new(path).exists() {
            println!("File exists. Resuming.");
            file = Some(
                std::fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(path)
                    .unwrap(),
            );

            let file_size = std::fs::metadata(path).unwrap().len();
            downloaded = file_size;
        } else {
            println!("Writing to new file.");
            file = Some(
                File::create(path)
                    .or(Err(format!("Failed to create file '{}'", path)))
                    .unwrap(),
            );
        }
    }
    (file, downloaded)
}

pub fn get_output(path: &str) -> (Box<dyn Write>, u64) {
    let (file, downloaded) = get_file(path);
    let output: Box<dyn Write> = Box::new(std::io::BufWriter::new(match path.len() {
        0 => Box::new(std::io::stdout()) as Box<dyn Write>,
        _ => Box::new(file.unwrap()) as Box<dyn Write>,
    }));

    (output, downloaded)
}
