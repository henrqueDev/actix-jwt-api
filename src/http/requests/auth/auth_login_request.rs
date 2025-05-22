use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

static RE_ONLY_DIGITS: LazyLock<Regex> = LazyLock::new(|| 
    Regex::new(r"^[0-9]*$").unwrap()
);

#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct AuthLoginRequest {
    #[validate(required, email(message="You have to insert a valid email!"), length(min=1, message="Insert a email!"))]
    pub email: Option<String>,
    #[validate(required, length(min=1, message="Insert a password!"))]
    pub password: Option<String>,
    #[validate(
        length(equal=6, message="2FA code must be equal 6 digits!"),
        regex(path="RE_ONLY_DIGITS", message = "2FA code must be only digits!")
    )]
    pub code: Option<String>
}