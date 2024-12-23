use std::fmt::{self, Display};
use std::str::FromStr;

use crate::utils::error_handling::{Error, Result};
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::{Multipart, Path};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{get, post, RouterIntoService};
use axum::Router;
use html_escape::encode_quoted_attribute;
use indoc::formatdoc;
use serde::{de, ser, Deserialize, Serialize};

pub fn routes() -> RouterIntoService<Body> {
    Router::new()
        .route("/star", get(star))
        .route("/present/:color", get(present))
        .route("/ornament/:state/:id", get(ornament))
        .route("/lockfile", post(lockfile))
        .into_service()
}

async fn star() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(r#"<div id="star" class="lit"></div>"#.into())?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Color {
    Red,
    Blue,
    Purple,
}

impl TryFrom<String> for Color {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "red" => Ok(Color::Red),
            "purple" => Ok(Color::Purple),
            "blue" => Ok(Color::Blue),
            _ => Err(anyhow!("Invalid color").into()),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Color::Red => "red",
            Color::Blue => "blue",
            Color::Purple => "purple",
        };
        write!(f, "{}", s)
    }
}

impl Color {
    fn next(&self) -> Self {
        match self {
            Color::Red => Color::Blue,
            Color::Blue => Color::Purple,
            Color::Purple => Color::Red,
        }
    }
}

async fn present(Path(color): Path<String>) -> Result<Response> {
    let Ok(color) = Color::try_from(color) else {
        return Ok(Response::builder()
            .status(StatusCode::IM_A_TEAPOT)
            .body(Body::empty())?);
    };
    let next_color = color.next();
    let present = formatdoc! {r#"
      <div class="present {color}" hx-get="/23/present/{next_color}" hx-swap="outerHTML">
        <div class="ribbon"></div>
        <div class="ribbon"></div>
        <div class="ribbon"></div>
        <div class="ribbon"></div>
      </div>
    "#};
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(present.into())?)
}

async fn ornament(Path((state_str, id)): Path<(String, String)>) -> Result<Response> {
    let state = match state_str.to_lowercase().as_str() {
        "on" => true,
        "off" => false,
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::IM_A_TEAPOT)
                .body(Body::empty())?)
        }
    };

    let ornament = formatdoc! {r#"
      <div class="{class}" id="{id}" hx-trigger="load delay:2s once" hx-get="{hx_get}" hx-swap="outerHTML"></div>
    "#,
      class = format!("ornament{}", if state { " on" } else { "" }),
      id = format!("ornament{id}", id = encode_quoted_attribute(&id)),
      hx_get = format!("/23/ornament/{next_state}/{id}", id = encode_quoted_attribute(&id), next_state = if state { "off" } else { "on" }),
    };
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(ornament.into())?)
}

/// Parsed Cargo.lock file containing dependencies
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Lockfile {
    /// Dependencies enumerated in the lockfile
    pub package: Vec<Package>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, PartialOrd, Ord, Hash)]
pub struct Package {
    /// Checksum for this package
    pub checksum: Option<Checksum>,
}

/// Cryptographic checksum (SHA-256) for a package
#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Checksum {
    /// SHA-256 digest of a package
    Sha256([u8; 32]),
}

impl From<[u8; 32]> for Checksum {
    fn from(bytes: [u8; 32]) -> Checksum {
        Checksum::Sha256(bytes)
    }
}

impl FromStr for Checksum {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Validate input is non-empty, has even length, and minimum length of 8 characters (4 bytes)
        if s.is_empty() || s.len() % 2 != 0 || s.len() < 8 {
            return Err(anyhow!("invalid_checksum").into());
        }

        // Validate that input only contains valid hex characters
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow!("invalid_checksum").into());
        }

        let mut digest = [0u8; 32];

        // Handle shorter checksums by only parsing available characters
        let chars = s.len();
        let bytes = chars / 2;
        for i in 0..bytes {
            digest[i] = u8::from_str_radix(&s[(i * 2)..=(i * 2) + 1], 16)?;
        }

        // Zero-pad the remaining bytes
        for i in bytes..32 {
            digest[i] = 0;
        }

        Ok(Checksum::Sha256(digest))
    }
}

impl fmt::Debug for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Checksum::Sha256(_) => write!(f, "Sha256({:x})", self),
        }
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::LowerHex for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Checksum::Sha256(digest) => {
                for b in digest {
                    write!(f, "{:02x}", b)?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::UpperHex for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Checksum::Sha256(digest) => {
                for b in digest {
                    write!(f, "{:02X}", b)?;
                }
            }
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for Checksum {
    fn deserialize<D: de::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        hex.parse().map_err(de::Error::custom)
    }
}

impl Serialize for Checksum {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

async fn lockfile(mut multipart: Multipart) -> Result<Response> {
    let mut lockfile_bytes = vec![];

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().map(|s| s.to_string()).unwrap_or_default();
        if name != "lockfile" {
            continue;
        }
        let data = field.bytes().await?;
        lockfile_bytes.extend(data);
    }

    if lockfile_bytes.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())?);
    }

    let lockfile_str = String::from_utf8(lockfile_bytes).map_err(|e| anyhow!(e.to_string()))?;

    let mut lockfile: Option<Lockfile> = None;
    match toml::from_str::<Lockfile>(&lockfile_str) {
        Ok(parsed) => {
            lockfile = Some(parsed);
        }
        Err(e) => match e.message() {
            "invalid_checksum" => {
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(Body::empty())?);
            }
            _ => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::empty())?);
            }
        },
    }

    let Some(packages) = lockfile.map(|lockfile| lockfile.package) else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())?);
    };

    let dots = packages
        .iter()
        .filter_map(|p| {
            let Some(checksum) = &p.checksum else {
                return None;
            };
            let (color, top, left) = match checksum {
                Checksum::Sha256(bytes) => (
                    format!("#{:02x}{:02x}{:02x}", bytes[0], bytes[1], bytes[2]),
                    format!("{:.2}", bytes[3]),
                    format!("{:.2}", bytes[4]),
                ),
            };
            Some(formatdoc! {r#"
              <div style="background-color:{color};top:{top}px;left:{left}px;"></div>
            "#})
        })
        .collect::<Vec<_>>();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(dots.join("\n").into())?)
}
