use axum::http::StatusCode;
use axum::response::Response;

pub fn invalid_manifest() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("Invalid manifest".into())
        .unwrap()
}

pub fn magic_keyword_not_provided() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("Magic keyword not provided".into())
        .unwrap()
}

pub fn no_content() -> Response {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body("".into())
        .unwrap()
}

pub fn unsupported_content_type() -> Response {
    Response::builder()
        .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
        .body("".into())
        .unwrap()
}
