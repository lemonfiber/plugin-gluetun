//! Gluetun, the egress guard, asked through its control server with the key lemonfiber
//! wrote for this adapter, and held to that key by the role file Gluetun reads when it
//! starts.
//!
//! Gluetun answers every route of its control server to anyone until it reads a role
//! file. The one this adapter writes names a single role, under the key, for the two
//! routes the contract reads, so every other route, and these two without the key, are
//! refused. The file is written owner-only through the shared directory, so a link or a
//! file that is not plain is replaced and never written through.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use lemonfiber_adapter_kit::credential::Holding;
use lemonfiber_adapter_kit::endpoint::Endpoint;
use lemonfiber_adapter_kit::shared::{Shared, SharedName, Unshared};
use lemonfiber_ports::http::{Http, Method, Request};
use lemonfiber_ports::service::{Applied, Failure, Tunnelling};
use serde::{Deserialize, Serialize};

/// What every failure is blamed on.
pub const SERVICE: &str = "gluetun";

/// The role file Gluetun reads from its configuration directory when it starts.
pub const ROLES: &str = "config.toml";

/// The header Gluetun reads a role's key from.
const KEY_HEADER: &str = "X-API-Key";

/// Where Gluetun says whether its tunnel runs.
const STATUS: &str = "/v1/vpn/status";

/// Where Gluetun says the ports its provider forwards.
const FORWARDED: &str = "/v1/portforward";

/// The status of a tunnel that is up.
const RUNNING: &str = "running";

/// What the role this adapter writes is named, in Gluetun's logs.
const ROLE: &str = "lemonfiber";

/// The way the role proves itself: the key, in [`KEY_HEADER`].
const BY_KEY: &str = "apikey";

/// The method of every route the role opens.
const GET: &str = "GET";

/// The mode the role file is written with: it carries the key.
const MODE: u32 = 0o600;

/// What Gluetun answers a request no role admits.
const UNAUTHORISED: u16 = 401;

/// Whether the tunnel runs, as Gluetun says it.
#[derive(Deserialize)]
struct Status {
    status: String,
}

/// The ports Gluetun's provider forwards, none where it forwards nothing.
#[derive(Deserialize)]
struct Forwarded {
    #[serde(default)]
    ports: Option<Vec<u16>>,
}

/// The role file, as Gluetun reads it.
#[derive(Serialize)]
struct Roles<'a> {
    roles: [Role<'a>; 1],
}

/// One role: its name, the routes it opens, and the key it proves itself with.
#[derive(Serialize)]
struct Role<'a> {
    name: &'a str,
    routes: [String; 2],
    auth: &'a str,
    apikey: &'a str,
}

/// One Gluetun, asked through its control server with the key `held` supplies.
pub struct Gluetun {
    endpoint: Endpoint,
    held: Arc<dyn Holding>,
    directory: PathBuf,
}

impl Gluetun {
    /// The Gluetun whose control server answers at `base` over `http`, asked with the key
    /// `held` supplies, whose configuration directory is shared at `directory`.
    #[must_use]
    pub fn new(
        http: Arc<dyn Http>,
        base: impl Into<String>,
        held: Arc<dyn Holding>,
        directory: &Path,
    ) -> Self {
        Self {
            endpoint: Endpoint::new(http, base, SERVICE),
            held,
            directory: directory.to_path_buf(),
        }
    }

    /// Whether Gluetun's control server answers at all, with or without admitting the
    /// request.
    pub async fn answers(&self) -> bool {
        self.endpoint.send(&self.asked(STATUS, None)).await.is_ok()
    }

    /// The key held now.
    fn key(&self) -> Result<String, Failure> {
        self.held.key().map_err(|why| {
            self.endpoint.refused(&format!(
                "this adapter holds no key for Gluetun's control server: {why}"
            ))
        })
    }

    /// A request for `path`, carrying `key` where there is one.
    fn asked(&self, path: &str, key: Option<&str>) -> Request {
        Request {
            method: Method::Get,
            url: self.endpoint.url(path),
            headers: key
                .map(|key| vec![(KEY_HEADER.to_owned(), key.to_owned())])
                .unwrap_or_default(),
            body: None,
            pinned: None,
        }
    }

    /// Whether Gluetun runs under the role file: it admits a request under `key` and
    /// refuses one without it.
    async fn holds(&self, key: &str) -> bool {
        let keyed = self.endpoint.send(&self.asked(STATUS, Some(key))).await;
        let bare = self.endpoint.send(&self.asked(STATUS, None)).await;
        matches!(
            (keyed, bare),
            (Ok(keyed), Ok(bare)) if keyed.is_success() && bare.status == UNAUTHORISED
        )
    }
}

#[async_trait]
impl Tunnelling for Gluetun {
    async fn up(&self) -> Result<bool, Failure> {
        let key = self.key()?;
        let response = self.endpoint.send(&self.asked(STATUS, Some(&key))).await?;
        let said: Status = self
            .endpoint
            .decode(&response, "the tunnel's status could not be read")?;
        Ok(said.status == RUNNING)
    }

    async fn forwarded(&self) -> Result<Option<u16>, Failure> {
        let key = self.key()?;
        let response = self
            .endpoint
            .send(&self.asked(FORWARDED, Some(&key)))
            .await?;
        let said: Forwarded = self
            .endpoint
            .decode(&response, "the forwarded ports could not be read")?;
        Ok(said
            .ports
            .unwrap_or_default()
            .into_iter()
            .find(|port| *port != 0))
    }

    async fn secure(&self) -> Result<Applied, Failure> {
        let key = self.key()?;
        let written = roles(&key);
        let shared = Shared::at(&self.directory);
        let name = SharedName::new(ROLES).map_err(unshared)?;
        let held = match shared.read(&name).await {
            Ok(held) => held,
            Err(Unshared::Failed(..)) => return Err(unavailable()),
            Err(_) => None,
        };
        if held.as_deref() != Some(written.as_bytes()) {
            shared
                .write(&name, written.into_bytes(), MODE)
                .await
                .map_err(unshared)?;
            return Ok(Applied::OnRestart);
        }
        Ok(if self.holds(&key).await {
            Applied::Now
        } else {
            Applied::OnRestart
        })
    }
}

/// The role file opening the two routes the contract reads, as Gluetun names a route,
/// to `key` and nothing else.
fn roles(key: &str) -> String {
    toml::to_string(&Roles {
        roles: [Role {
            name: ROLE,
            routes: [format!("{GET} {STATUS}"), format!("{GET} {FORWARDED}")],
            auth: BY_KEY,
            apikey: key,
        }],
    })
    .unwrap_or_default()
}

/// Gluetun's configuration directory not there to read or write.
fn unavailable() -> Failure {
    Failure::Unavailable {
        service: SERVICE.to_owned(),
    }
}

/// The role file that could not be written: a directory that is not there leaves
/// Gluetun unavailable, and one that is a link is refused by name.
fn unshared(why: Unshared) -> Failure {
    match why {
        Unshared::Failed(..) => unavailable(),
        refusal => Failure::Refused {
            service: SERVICE.to_owned(),
            detail: refusal.to_string(),
        },
    }
}

#[cfg(test)]
mod tests;
