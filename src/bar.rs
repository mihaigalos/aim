use indicatif::{ProgressBar, ProgressStyle};

pub fn get_progress_bar(total_size: u64, url: &str) -> indicatif::ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("█  "));
    pb.set_message(&format!("⛵ Downloading {}", url));
    pb
}
