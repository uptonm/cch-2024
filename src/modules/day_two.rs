use axum::extract::Query;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

use crate::utils::error_handling::Result;
use crate::utils::network_address::{IPv4Addr, IPv6Addr};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dest", get(egregious_encryption))
        .route("/key", get(egregious_decryption))
        .route("/v6/dest", get(egregious_encryption_v6))
        .route("/v6/key", get(egregious_decryption_v6))
}

#[derive(Debug, Deserialize)]
struct EncryptParams {
    from: String,
    key: String,
}

#[derive(Debug, Deserialize)]
struct DecryptParams {
    from: String,
    to: String,
}

async fn egregious_encryption(
    Query(EncryptParams { from, key }): Query<EncryptParams>,
) -> Result<String> {
    let from = IPv4Addr::try_from(from)?;
    let key = IPv4Addr::try_from(key)?;
    let result = from.wrapping_add(&key)?;
    Ok(result.into())
}

async fn egregious_decryption(
    Query(DecryptParams { from, to }): Query<DecryptParams>,
) -> Result<String> {
    let from = IPv4Addr::try_from(from)?;
    let to = IPv4Addr::try_from(to)?;
    let result = from.wrapping_sub(&to)?;
    Ok(result.into())
}

async fn egregious_encryption_v6(
    Query(EncryptParams { from, key }): Query<EncryptParams>,
) -> Result<String> {
    let from = IPv6Addr::try_from(from)?;
    let key = IPv6Addr::try_from(key)?;
    let result = from.xor(&key);
    Ok(result.into())
}

async fn egregious_decryption_v6(
    Query(DecryptParams { from, to }): Query<DecryptParams>,
) -> Result<String> {
    let from = IPv6Addr::try_from(from)?;
    let to = IPv6Addr::try_from(to)?;
    let result = from.xor(&to);
    Ok(result.into())
}
