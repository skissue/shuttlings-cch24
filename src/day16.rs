use jsonwebtoken::{errors::ErrorKind as JWTError, DecodingKey, EncodingKey, Header, Validation};
use poem::{
    get, handler,
    http::{HeaderMap, StatusCode},
    post, Response, Route,
};
use std::collections::HashSet;

pub fn route() -> Route {
    Route::new()
        .at("/wrap", post(wrap_gift))
        .at("/unwrap", get(unwrap_gift))
        .at("/decode", post(decode_old_gift))
}

#[handler]
fn wrap_gift(body: String) -> Response {
    let jwt = jsonwebtoken::encode(
        &Header::default(),
        &serde_json::from_str::<serde_json::Value>(&body).expect("Input is valid JSON"),
        &EncodingKey::from_secret(b"a"),
    )
    .expect("Failed to encode JWT");

    Response::builder()
        .header("Set-Cookie", format!("gift={jwt}",))
        .body(())
}

#[handler]
async fn unwrap_gift(headers: &HeaderMap) -> Response {
    let Some(cookie) = headers
        .get("Cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|c| c.strip_prefix("gift="))
    else {
        return StatusCode::BAD_REQUEST.into();
    };

    let mut validation = Validation::default();
    validation.required_spec_claims = HashSet::new();
    validation.validate_exp = false;

    let Ok(decoded) = jsonwebtoken::decode::<serde_json::Value>(
        cookie,
        &DecodingKey::from_secret(b"a"),
        &validation,
    )
    .map(|d| d.claims) else {
        return StatusCode::BAD_REQUEST.into();
    };

    decoded.to_string().into()
}

#[handler]
async fn decode_old_gift(body: String) -> Response {
    let mut validation = Validation::default();
    validation.required_spec_claims = HashSet::new();
    validation.validate_exp = false;
    validation.algorithms = vec![
        jsonwebtoken::Algorithm::RS256,
        jsonwebtoken::Algorithm::RS512,
    ];

    let decoded = match jsonwebtoken::decode::<serde_json::Value>(
        &body,
        &DecodingKey::from_rsa_pem(include_bytes!("../day16_santa_public_key.pem"))
            .expect("Key from filesystem is valid"),
        &validation,
    )
    .map(|c| c.claims)
    {
        Ok(decoded) => decoded,
        Err(err) => {
            return (match err.into_kind() {
                JWTError::InvalidSignature => StatusCode::UNAUTHORIZED,
                _ => StatusCode::BAD_REQUEST,
            })
            .into()
        }
    };
    decoded.to_string().into()
}
