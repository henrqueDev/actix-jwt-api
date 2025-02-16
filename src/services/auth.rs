use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{Duration, UNIX_EPOCH};

use crate::model::user::UserDTO;

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    custom_claim: String,
    iss: String,
    sub: String,
    aud: String,
    exp: u64,
}

pub fn encode_jwt(user: UserDTO) -> String {
    let now_plus_60 = std::time::SystemTime::now()
        .checked_add(Duration::from_secs(6120))
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let c = Claims {
        custom_claim: "".to_owned(),
        iss: "http://0.0.0.0".to_owned(),
        sub: user.email.to_owned(),
        exp: now_plus_60,
        aud: "pethotel-api".to_owned(),
    };

    let header = jsonwebtoken::Header::default();

    let secret = jsonwebtoken::EncodingKey::from_secret(user.password.as_bytes());

    let res = jsonwebtoken::encode(&header, &c, &secret).unwrap();
    return res;
}

pub fn decode_jwt(res: &str, secret: &jsonwebtoken::DecodingKey) {
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    // skip exp validation, which is on by default
    validation.required_spec_claims = HashSet::new();

    // skip aud validation
    validation.validate_aud = false;

    // decode token
    let res = jsonwebtoken::decode::<Claims>(res, secret, &validation);
    println!("res {:?}", res);

    // token result
    if let Ok(token_data) = res {
        println!("token_data: {:?}", token_data);
        // Accessing the decoded information
        let claims = token_data.claims;
        println!("decoded claims: {:?}", claims);
    } else {
        println!("Error decoding the token");
    }
}
