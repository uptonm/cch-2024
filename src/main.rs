use axum::Router;

mod day_negative_one {
    use axum::{
        http::{header, StatusCode},
        response::IntoResponse,
        routing::get,
        Router,
    };

    async fn hello_bird() -> &'static str {
        "Hello, bird!"
    }

    const REDIRECT_URL: &str = "https://www.youtube.com/watch?v=9Gc4QTqslN4";

    async fn seek() -> impl IntoResponse {
        (StatusCode::FOUND, [(header::LOCATION, REDIRECT_URL)])
    }

    pub fn routes() -> Router {
        Router::new()
            .route("/", get(hello_bird))
            .route("/-1/seek", get(seek))
    }
}

mod day_two {
    use axum::{extract::Query, routing::get, Router};
    use serde::Deserialize;

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

    struct IPv4Addr {
        octets: [u8; 4],
    }

    impl IPv4Addr {
        fn wrapping_add(&self, other: &Self) -> Self {
            Self {
                octets: self
                    .octets
                    .into_iter()
                    .zip(other.octets.into_iter())
                    .map(|(a, b)| a.wrapping_add(b))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            }
        }

        fn wrapping_sub(&self, other: &Self) -> Self {
            Self {
                octets: self
                    .octets
                    .into_iter()
                    .zip(other.octets.into_iter())
                    .map(|(a, b)| b.wrapping_sub(a))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            }
        }
    }

    impl From<String> for IPv4Addr {
        fn from(s: String) -> Self {
            let addr = s.parse::<std::net::Ipv4Addr>().unwrap();
            Self {
                octets: addr.octets(),
            }
        }
    }

    impl From<IPv4Addr> for String {
        fn from(addr: IPv4Addr) -> Self {
            let addr = std::net::Ipv4Addr::from(addr.octets);
            addr.to_string()
        }
    }

    async fn egregious_encryption(
        Query(EncryptParams { from, key }): Query<EncryptParams>,
    ) -> String {
        let from = IPv4Addr::from(from);
        let key = IPv4Addr::from(key);
        let result = from.wrapping_add(&key);
        result.into()
    }

    async fn egregious_decryption(
        Query(DecryptParams { from, to }): Query<DecryptParams>,
    ) -> String {
        let from = IPv4Addr::from(from);
        let to = IPv4Addr::from(to);
        let result = from.wrapping_sub(&to);
        result.into()
    }

    struct IPv6Addr {
        octets: [u16; 8],
    }

    impl IPv6Addr {
        fn xor(&self, other: &Self) -> Self {
            Self {
                octets: self
                    .octets
                    .into_iter()
                    .zip(other.octets.into_iter())
                    .map(|(a, b)| a ^ b)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            }
        }
    }

    impl From<String> for IPv6Addr {
        fn from(s: String) -> Self {
            let addr = s.parse::<std::net::Ipv6Addr>().unwrap();
            Self {
                octets: addr.segments(),
            }
        }
    }

    impl From<IPv6Addr> for String {
        fn from(addr: IPv6Addr) -> Self {
            let addr = std::net::Ipv6Addr::from(addr.octets);
            addr.to_string()
        }
    }

    async fn egregious_encryption_v6(
        Query(EncryptParams { from, key }): Query<EncryptParams>,
    ) -> String {
        let from = IPv6Addr::from(from);
        let key = IPv6Addr::from(key);
        let result = from.xor(&key);
        result.into()
    }

    async fn egregious_decryption_v6(
        Query(DecryptParams { from, to }): Query<DecryptParams>,
    ) -> String {
        let from = IPv6Addr::from(from);
        let to = IPv6Addr::from(to);
        let result = from.xor(&to);
        result.into()
    }

    pub fn routes() -> Router {
        Router::new()
            .route("/dest", get(egregious_encryption))
            .route("/key", get(egregious_decryption))
            .route("/v6/dest", get(egregious_encryption_v6))
            .route("/v6/key", get(egregious_decryption_v6))
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .nest("/", day_negative_one::routes())
        .nest("/2", day_two::routes());
    Ok(router.into())
}
