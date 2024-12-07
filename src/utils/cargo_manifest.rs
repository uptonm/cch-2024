use core::fmt;
use std::fmt::Display;

use axum::extract::{FromRequest, Request};
use axum::http::header::CONTENT_TYPE;
use axum::response::Response;
use axum::RequestExt;
use cargo_manifest::Manifest;
use serde::Deserialize;

use crate::utils::error_responses::{
    invalid_manifest, magic_keyword_not_provided, no_content, unsupported_content_type,
};

#[serde_with::serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct Order {
    #[serde(rename = "item")]
    pub item: String,
    #[serde_as(deserialize_as = "serde_with::DefaultOnError")]
    #[serde(default, rename = "quantity")]
    pub quantity: u32,
}

impl Order {
    pub fn new(item: String, quantity: u32) -> Self {
        Self { item, quantity }
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.item, self.quantity)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Metadata {
    #[serde(default, rename = "orders")]
    pub orders: Vec<Order>,
}

impl Metadata {
    pub fn new(orders: Vec<Order>) -> Self {
        Self { orders }
    }

    pub fn add_order(&mut self, item: String, quantity: u32) {
        self.orders.push(Order::new(item, quantity));
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.orders
                .iter()
                .map(|o| o.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[async_trait::async_trait]
impl<S> FromRequest<S> for Metadata {
    type Rejection = Response;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = req.headers().clone();
        let Some(content_type) = headers.get(CONTENT_TYPE) else {
            return Err(unsupported_content_type());
        };
        let body: String = req.extract().await.map_err(|_| invalid_manifest())?;

        let parsed_manifest: Manifest;

        match content_type.to_str() {
            Ok(content_type) => match content_type {
                "application/toml" => match toml::from_str(&body) {
                    Ok(manifest) => parsed_manifest = manifest,
                    Err(_) => return Err(invalid_manifest()),
                },
                "application/json" => match serde_json::from_str(&body) {
                    Ok(manifest) => parsed_manifest = manifest,
                    Err(_) => return Err(invalid_manifest()),
                },
                "application/yaml" => match serde_yaml::from_str(&body) {
                    Ok(manifest) => parsed_manifest = manifest,
                    Err(_) => return Err(invalid_manifest()),
                },
                _ => return Err(unsupported_content_type()),
            },
            Err(_) => return Err(unsupported_content_type()),
        }

        let Some(package) = parsed_manifest.package else {
            return Err(magic_keyword_not_provided());
        };

        let Some(cargo_manifest::MaybeInherited::Local(keywords)) = package.keywords else {
            return Err(magic_keyword_not_provided());
        };

        if !keywords.contains(&"Christmas 2024".to_string()) {
            return Err(magic_keyword_not_provided());
        }

        let Some(metadata) = package.metadata else {
            return Err(no_content());
        };

        let cargo_manifest::Value::Table(metadata) = metadata else {
            return Err(no_content());
        };

        let Some(cargo_manifest::Value::Array(orders)) = metadata.get("orders") else {
            return Err(no_content());
        };

        let mut metadata = Metadata::new(vec![]);
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

            let Ok(quantity) = u32::try_from(*quantity) else {
                continue;
            };

            metadata.add_order(item.clone(), quantity);
        }

        Ok(metadata)
    }
}
