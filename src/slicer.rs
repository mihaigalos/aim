pub struct Slicer;
impl Slicer {
    pub fn target_with_extension(s: &str) -> &str {
        let pos_of_last_slash = s.rfind('/').unwrap();
        &s[pos_of_last_slash + 1..]
    }

    pub fn target_without_extension(s: &str) -> &str {
        let target_with_extension = Slicer::target_with_extension(s);
        if s.ends_with(".tar.gz") {
            let pos_extension = target_with_extension.find(".tar.gz").unwrap();
            &target_with_extension[..pos_extension]
        } else {
            let pos_of_last_dot = target_with_extension.rfind('.').unwrap();
            &target_with_extension[..pos_of_last_dot]
        }
    }

    pub fn target(s: &str) -> &str {
        let target_with_extension = Slicer::target_with_extension(s);
        let pos_of_first_dash = target_with_extension.find('-').unwrap();
        &target_with_extension[..pos_of_first_dash]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_with_extension() {
        let is = Slicer::target_with_extension("https://github.com/Byron/dua-cli/releases/download/v2.10.2/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz");
        let expected = "dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz";
        assert_eq!(is, expected);
    }
    #[test]
    fn target_without_extension_tar_gz() {
        let is = Slicer::target_without_extension("https://github.com/Byron/dua-cli/releases/download/v2.10.2/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz");
        let expected = "dua-v2.10.2-x86_64-unknown-linux-musl";
        assert_eq!(is, expected);
    }
    #[test]
    fn target_without_extension_zip() {
        let is = Slicer::target_without_extension(
            "https://github.com/ogham/exa/releases/download/v0.9.0/exa-linux-x86_64-0.9.0.zip",
        );
        let expected = "exa-linux-x86_64-0.9.0";
        assert_eq!(is, expected);
    }
    #[test]
    fn target() {
        let is = Slicer::target("https://github.com/Byron/dua-cli/releases/download/v2.10.2/dua-v2.10.2-x86_64-unknown-linux-musl.tar.gz");
        let expected = "dua";
        assert_eq!(is, expected);
    }
}
