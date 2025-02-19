use serde::{Deserialize, Serialize};
use std::time::{Duration, UNIX_EPOCH};
use dotenv_codegen::dotenv;
use crate::model::user::UserDTO;

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: u64,
}

pub fn encode_jwt(user: UserDTO) -> String {
    let app_name = dotenv!("APP_NAME");
    let app_addr = dotenv!("APP_ADDRESS");
    let secret = dotenv!("SECRET_KEY");

    let time_exp = std::time::SystemTime::now()
        .checked_add(Duration::from_secs(3200))
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let c = Claims {
        iss: app_addr.to_owned(),
        sub: user.email.to_owned(),
        exp: time_exp,
        aud: app_name.to_owned(),
    };

    let header = jsonwebtoken::Header::default();
    let key_encoded = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());

    let res = jsonwebtoken::encode(&header, &c, &key_encoded).unwrap();
    return res;
}

pub fn decode_jwt(res: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let app_name = dotenv!("APP_NAME");
    let app_addr = dotenv!("APP_ADDRESS");
    let secret = dotenv!("SECRET_KEY");

    let secret_key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    
    //validation.required_spec_claims = HashSet::new();

    validation.set_audience(&[&app_name]);
    validation.set_issuer(&[&app_addr]);

    let res = match jsonwebtoken::decode::<Claims>(res, &secret_key, &validation) {
        Ok(token) => Ok(token.claims),
        Err(err) => Err(err)
    };
    
    return res;
}
