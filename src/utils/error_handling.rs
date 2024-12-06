use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
