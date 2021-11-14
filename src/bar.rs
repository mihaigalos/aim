use dotenv::dotenv;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;

const DEFAULT_SHIP_PROGRESSBAR_TEMPLATE: &str = "{msg}\n{spinner:.cyan}  {elapsed_precise} ▕{bar:.white}▏ {bytes}/{total_bytes}  {bytes_per_sec}  ETA {eta}.";
const DEFAULT_SHIP_PROGRESSBAR_PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏  ";

fn construct_progress_bar(
    total_size: u64,
    url: &str,
    template: &str,
    progress_chars: &str,
) -> indicatif::ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_message(format!("⛵ Downloading {}", url.clone()));
    pb.set_style(
        ProgressStyle::default_bar()
            .template(template)
            .progress_chars(progress_chars),
    );
    pb
}
pub fn get_progress_bar(
    total_size: u64,
    url: &str,
    silent: bool,
) -> Option<indicatif::ProgressBar> {
    dotenv().ok();

    let template = env::var("SHIP_PROGRESSBAR_TEMPLATE")
        .unwrap_or(DEFAULT_SHIP_PROGRESSBAR_TEMPLATE.to_string());

    let progress_chars = env::var("SHIP_PROGRESSBAR_PROGRESS_CHARS")
        .unwrap_or(DEFAULT_SHIP_PROGRESSBAR_PROGRESS_CHARS.to_string());

    let result = match silent {
        false => Some(construct_progress_bar(
            total_size,
            url,
            &template,
            &progress_chars,
        )),
        true => None,
    };
    result
}
