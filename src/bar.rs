use dotenv::dotenv;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
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
    pb.set_draw_target(ProgressDrawTarget::hidden());
    pb.set_message(format!("⛵ Downloading {}", url.clone()));
    pb.set_style(
        ProgressStyle::default_bar()
            .template(template)
            .progress_chars(progress_chars),
    );
    pb
}

pub struct WrappedBar {
    pub silent: bool,
    output: Option<indicatif::ProgressBar>,
}

impl WrappedBar {
    pub fn new_empty() -> WrappedBar {
        WrappedBar {
            silent: true,
            output: None,
        }
    }
    pub fn new(total_size: u64, url: &str, silent: bool) -> WrappedBar {
        dotenv().ok();

        let template = &env::var("SHIP_PROGRESSBAR_TEMPLATE")
            .unwrap_or(DEFAULT_SHIP_PROGRESSBAR_TEMPLATE.to_string());

        let progress_chars = &env::var("SHIP_PROGRESSBAR_PROGRESS_CHARS")
            .unwrap_or(DEFAULT_SHIP_PROGRESSBAR_PROGRESS_CHARS.to_string());
        let output = match silent {
            false => Some(construct_progress_bar(
                total_size,
                url,
                template,
                progress_chars,
            )),
            true => None,
        };

        WrappedBar {
            silent: silent,
            output: output,
        }
    }
    pub fn set_length(&self, len: u64) {
        if !self.silent {
            self.output
                .as_ref()
                .unwrap()
                .set_draw_target(ProgressDrawTarget::stderr());
            self.output.as_ref().unwrap().set_length(len);
        }
    }
    pub fn set_position(&self, pos: u64) {
        if !self.silent {
            self.output.as_ref().unwrap().set_position(pos);
        }
    }
    pub fn finish_with_message(&self, msg: String) {
        if !self.silent {
            self.output.as_ref().unwrap().finish_with_message(msg);
        }
    }
}
