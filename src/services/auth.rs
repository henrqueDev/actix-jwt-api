use serde::{Deserialize, Serialize};
use std::time::{Duration, UNIX_EPOCH};
use dotenvy_macro::dotenv;

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    iss: String,
    pub sub: String,
    aud: String,
    exp: u64,
}

pub fn encode_jwt(user_email: String) -> String {
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
        sub: user_email,
        exp: time_exp,
        aud: app_name.to_owned(),
    };

    let key_encoded = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());

    return jsonwebtoken::encode(
        &jsonwebtoken::Header::default(), 
        &c, 
        &key_encoded
    ).unwrap();
}

pub fn decode_jwt(res: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let app_name = dotenv!("APP_NAME");
    let app_addr = dotenv!("APP_ADDRESS");
    let secret = dotenv!("SECRET_KEY");

    let secret_key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

    validation.set_audience(&[&app_name]);
    validation.set_issuer(&[&app_addr]);

    let res = match jsonwebtoken::decode::<Claims>(res, &secret_key, &validation) {
        Ok(token) => Ok(token.claims),
        Err(err) => Err(err)
    };
    
    return res;
}
