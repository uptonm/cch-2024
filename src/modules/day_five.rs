use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use cargo_manifest::Manifest;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct Metadata {
    #[serde(default, rename = "orders")]
    _orders: Vec<Order>,
}

#[serde_with::serde_as]
#[derive(Deserialize, Debug, Clone)]
struct Order {
    #[serde(rename = "item")]
    _item: String,
    #[serde_as(deserialize_as = "serde_with::DefaultOnError")]
    #[serde(default, rename = "quantity")]
    _quantity: u32,
}

pub fn routes() -> Router {
    Router::new().route("/manifest", post(manifest))
}

async fn manifest(headers: HeaderMap, body: String) -> Response {
    let Some(content_type) = headers.get(CONTENT_TYPE) else {
        return unsupported_content_type();
    };

    let parsed_manifest: Manifest;

    match content_type.to_str() {
        Ok(content_type) => match content_type {
            "application/toml" => match toml::from_str(&body) {
                Ok(manifest) => parsed_manifest = manifest,
                Err(_) => return invalid_manifest(),
            },
            "application/json" => match serde_json::from_str(&body) {
                Ok(manifest) => parsed_manifest = manifest,
                Err(_) => return invalid_manifest(),
            },
            "application/yaml" => match serde_yaml::from_str(&body) {
                Ok(manifest) => parsed_manifest = manifest,
                Err(_) => return invalid_manifest(),
            },
            _ => return unsupported_content_type(),
        },
        Err(_) => return unsupported_content_type(),
    }

    let Some(package) = parsed_manifest.package else {
        return magic_keyword_not_provided();
    };

    let Some(cargo_manifest::MaybeInherited::Local(keywords)) = package.keywords else {
        return magic_keyword_not_provided();
    };

    if !keywords.contains(&"Christmas 2024".to_string()) {
        return magic_keyword_not_provided();
    }

    let Some(metadata) = package.metadata else {
        return no_content();
    };

    let cargo_manifest::Value::Table(metadata) = metadata else {
        return no_content();
    };

    let Some(cargo_manifest::Value::Array(orders)) = metadata.get("orders") else {
        return no_content();
    };

    let mut result = vec![];
    for order in orders {
        let cargo_manifest::Value::Table(order) = order else {
            continue;
        };

        let Some(cargo_manifest::Value::String(item)) = order.get("item") else {
            continue;
        };

        let Some(cargo_manifest::Value::Integer(quantity)) = order.get("quantity") else {
            continue;
        };

        result.push(format!("{item}: {quantity}"));
    }

    if result.is_empty() {
        return no_content();
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(result.join("\n").into())
        .unwrap()
}

fn invalid_manifest() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("Invalid manifest".into())
        .unwrap()
}

fn magic_keyword_not_provided() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("Magic keyword not provided".into())
        .unwrap()
}

fn no_content() -> Response {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body("".into())
        .unwrap()
}

fn unsupported_content_type() -> Response {
    Response::builder()
        .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
        .body("".into())
        .unwrap()
}
