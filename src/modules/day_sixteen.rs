use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{get, post, RouterIntoService};
use axum::{Json, Router};

use axum_extra::extract::{cookie::Cookie, CookieJar};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde_json::Value;

pub fn routes() -> RouterIntoService<Body> {
    Router::new()
        .route("/wrap", post(wrap))
        .route("/unwrap", get(unwrap))
        .route("/decode", post(decode))
        .into_service()
}

// The secret is not private, as this is just a code-hunt...
const JWT_SECRET: &str = "SUPER_SECRET_KEY";

async fn wrap(jar: CookieJar, Json(claims): Json<Value>) -> (StatusCode, CookieJar) {
    let Ok(jwt) = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    ) else {
        return (StatusCode::BAD_REQUEST, jar);
    };
    (StatusCode::OK, jar.add(Cookie::new("gift", jwt)))
}

async fn unwrap(jar: CookieJar) -> Response {
    let Some(jwt) = jar.get("gift") else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap();
    };

    let mut validator = Validation::default();
    validator.required_spec_claims.clear();

    let Ok(TokenData { claims, .. }) = jsonwebtoken::decode::<Value>(
        jwt.value_trimmed(),
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &validator,
    ) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap();
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(claims.to_string()))
        .unwrap()
}

const SANTA_PEM: &str = include_str!("../../resources/santa.pem");

async fn decode(jwt: String) -> Response {
    let decoding_key = DecodingKey::from_rsa_pem(SANTA_PEM.trim().as_ref()).unwrap();
    let mut validation = Validation::new(Algorithm::RS256);
    validation.required_spec_claims.clear();
    validation.algorithms = vec![Algorithm::RS256, Algorithm::RS512];

    match jsonwebtoken::decode::<Value>(&jwt, &decoding_key, &validation) {
        Ok(TokenData { claims, .. }) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(claims.to_string()))
            .unwrap(),
        Err(error) => match error.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSignature => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap(),
            _ => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(error.to_string()))
                .unwrap(),
        },
    }
}
