use std::sync::LazyLock;

use serde::Deserialize;
use validator::Validate;
use regex::Regex;

static RE_ONLY_DIGITS: LazyLock<Regex> = LazyLock::new(|| 
    Regex::new(r"^[0-9]*$").unwrap()
);

#[derive(Debug, Deserialize, Validate)]
pub struct UserActivate2FARequest {
    #[validate(
        length(equal=6, message = "2FA code must be equal 6 digits!"), 
        regex(path="RE_ONLY_DIGITS", message = "2FA code must be only digits!")
    )]
    pub code: String
}