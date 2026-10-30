//! What this adapter says it is, and the kit that serves its contract.

use std::sync::Arc;

use lemonfiber_adapter_kit::{Kit, Readiness};
use lemonfiber_contract::adapter::{About, Release};
use lemonfiber_contract::capabilities::network::egress;

use crate::gluetun::Gluetun;

/// Where Gluetun's control server answers, by its service id and the port it listens on.
pub const ADDRESS: &str = "http://gluetun:8000";

/// Where Gluetun's configuration directory is, in the adapter's container, which the
/// adapter shares and Gluetun reads its control server's roles from.
pub const SHARED: &str = "/gluetun";

/// The part of `upstream.json` holding the key Gluetun's control server answers to.
pub const API_KEY: &str = "api_key";

/// The upstream this adapter fronts, as its project names itself.
pub const UPSTREAM: &str = "Gluetun";

/// The Gluetun release this adapter's recordings were taken from.
pub const RELEASE: &str = "v3.41.3";

/// The digest of the image that release's recordings were taken from.
pub const DIGEST: &str = "sha256:fa19cc76b2af13d57a8d3dc3066f2ada061b1c761b8aecf989b3877c0486e027";

/// What this adapter says it is: the contract it speaks and the release it supports.
#[must_use]
pub fn about() -> About {
    About {
        speaks: vec![lemonfiber_contract::spoken(
            egress::CAPABILITY,
            egress::MAJOR,
        )],
        upstream: UPSTREAM.to_owned(),
        releases: vec![Release {
            version: RELEASE.to_owned(),
            digest: DIGEST.to_owned(),
        }],
    }
}

/// The contract, served from `gluetun` under `key`, ready while its control server
/// answers.
#[must_use]
pub fn kit(key: String, gluetun: Arc<Gluetun>) -> Kit {
    let asked = Arc::clone(&gluetun);
    let ready: Readiness = Arc::new(move || {
        let asked = Arc::clone(&asked);
        Box::pin(async move { asked.answers().await })
    });
    Kit::new(key, about())
        .ready_when(ready)
        .serving(egress::served(gluetun))
}

#[cfg(test)]
mod tests;
