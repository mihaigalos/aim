extern crate custom_error;
use custom_error::custom_error;

custom_error! {
    pub ValidateError
    Sha256Mismatch = "Invalid sha256.",
}
