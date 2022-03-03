use regex::Regex;
use std::env;
use std::path::PathBuf;

/// Convert a tilde path to an absolute path: ~/Desktop → /Users/sathish/Desktop
///
/// Example
///
/// ```
/// use untildify::untildify;
///
/// fn main() {
///    println!("Untildify : {}", untildify::untildify("~/Desktop")); // prints /Users/<user_name>/Desktop
///    println!("Untildify : {}", untildify("~/a/b/c/d/e")); // prints "/User/Untildify/a/b/c/d/e"
///    println!("Untildify : {}", untildify("~/")); // prints "/User/Untildify/"
/// }
/// ```

pub fn untildify(input_path: &str) -> String {
    if input_path.is_empty() {
        return String::from(input_path);
    }
    return match get_host_dir() {
        Some(path) => {
            let host_dir = path.to_str().unwrap();
            let re = Regex::new(r"^~([/\w.]+)").unwrap();
            match re.captures(input_path) {
                Some(captures) => {
                    return format!("{}{}", host_dir, &captures[1]);
                }
                None => String::from(input_path),
            }
        }
        None => String::from(input_path),
    };
}

#[cfg(any(unix, target_os = "redox"))]
fn get_host_dir() -> Option<PathBuf> {
    #[allow(deprecated)]
    env::home_dir()
}

#[cfg(test)]
mod tests {
    use crate::untildify::untildify;
    use std::env;
    use std::path::Path;

    #[test]
    fn test_returns_untildfyed_string() {
        env::remove_var("HOME");

        let home = Path::new("/User/Untildify");
        env::set_var("HOME", home.as_os_str());

        assert_eq!(untildify("~/Desktop"), "/User/Untildify/Desktop");
        assert_eq!(untildify("~/a/b/c/d/e"), "/User/Untildify/a/b/c/d/e");
        assert_eq!(untildify("~/"), "/User/Untildify/");
    }

    #[test]
    fn test_returns_empty_string() {
        env::remove_var("HOME");

        let home = Path::new("/User/Untildify");
        env::set_var("HOME", home.as_os_str());

        assert_eq!(untildify("Desktop"), "Desktop");
        assert_eq!(untildify(""), "");
        assert_eq!(untildify("/"), "/");
        assert_eq!(untildify("~/Desktop/~/Code"), "/User/Untildify/Desktop/");
    }

    #[test]
    fn test_with_dot_folders() {
        env::remove_var("HOME");

        let home = Path::new("/User/Untildify");
        env::set_var("HOME", home.as_os_str());

        assert_eq!(untildify("~/.ssh/id_rsa"), "/User/Untildify/.ssh/id_rsa");
    }
}
