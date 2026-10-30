use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use lemonfiber_contract::adapter::{about_path, ready_path, About};
use lemonfiber_contract::capabilities::network::egress;
use lemonfiber_fixtures::http::{Answer, Fake};
use tower::ServiceExt as _;

use super::{about, kit, ADDRESS, DIGEST, RELEASE, UPSTREAM};
use crate::gluetun::Gluetun;

fn up() -> String {
    lemonfiber_contract::path(egress::CAPABILITY, egress::MAJOR, "up")
}

fn a_key() -> String {
    ('e'..='t').collect()
}

fn gluetun(fake: &Arc<Fake>) -> Arc<Gluetun> {
    Arc::new(Gluetun::new(
        fake.clone(),
        ADDRESS,
        Arc::new(('a'..='p').collect::<String>()),
        &std::env::temp_dir().join(format!("gluetun-adapter-{}", std::process::id())),
    ))
}

async fn answered_by(fake: &Arc<Fake>, path: &str, key: Option<&str>) -> (StatusCode, Vec<u8>) {
    let (method, body) = if path.starts_with("/lemonfiber/adapter/") {
        ("GET", "")
    } else {
        ("POST", "{}")
    };
    let mut request = Request::builder().method(method).uri(path);
    if let Some(key) = key {
        request = request.header(header::AUTHORIZATION, format!("Bearer {key}"));
    }
    let Ok(request) = request.body(Body::from(body)) else {
        return (StatusCode::IM_A_TEAPOT, Vec::new());
    };
    let Ok(response) = kit(a_key(), gluetun(fake)).router().oneshot(request).await;
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .map(|bytes| bytes.to_vec())
        .unwrap_or_default();
    (status, bytes)
}

#[test]
fn it_speaks_the_egress_guard_contract_and_names_the_release_its_recordings_were_taken_from() {
    let said = about();
    assert_eq!(said.speaks, ["network.egress-guard@1"]);
    assert_eq!(said.upstream, UPSTREAM);
    assert_eq!(
        said.releases
            .iter()
            .map(|release| (release.version.as_str(), release.digest.as_str()))
            .collect::<Vec<_>>(),
        [(RELEASE, DIGEST)]
    );
}

#[tokio::test]
async fn about_is_answered_under_the_plugin_s_key() {
    let key = a_key();
    let (status, body) = answered_by(&Fake::silent(), &about_path(), Some(&key)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(serde_json::from_slice::<About>(&body).ok(), Some(about()));
}

#[tokio::test]
async fn nothing_is_answered_without_the_key_or_under_another() {
    let fake = Fake::silent();
    for key in [None, Some("another")] {
        let (status, _) = answered_by(&fake, &up(), key).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    assert!(fake.requests().is_empty());
}

#[tokio::test]
async fn it_is_ready_while_the_control_server_answers_even_refusing() {
    let key = a_key();
    let refusing = Fake::always(Answer::reply(401, "Unauthorized"));
    let ready = answered_by(&refusing, &ready_path(), Some(&key)).await;
    let silent = answered_by(&Fake::silent(), &ready_path(), Some(&key)).await;
    assert_eq!(ready, (StatusCode::OK, b"true".to_vec()));
    assert_eq!(silent, (StatusCode::OK, b"false".to_vec()));
    assert!(refusing
        .requests()
        .iter()
        .all(|asked| asked.url == format!("{ADDRESS}/v1/vpn/status") && asked.headers.is_empty()));
}

#[tokio::test]
async fn the_tunnel_is_answered_over_the_contract() {
    let key = a_key();
    let running = Fake::always(Answer::reply(200, r#"{"status":"running"}"#));
    let (status, body) = answered_by(&running, &up(), Some(&key)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(serde_json::from_slice::<bool>(&body).ok(), Some(true));
}
