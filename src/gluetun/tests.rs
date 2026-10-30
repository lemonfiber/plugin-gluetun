use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::sync::Arc;

use lemonfiber_adapter_kit::credential::Holding;
use lemonfiber_fixtures::http::{Answer, Fake};
use lemonfiber_ports::http::Method;
use lemonfiber_ports::service::{Applied, Failure, Tunnelling};

use super::{Gluetun, ROLES};

const BASE: &str = "http://gluetun:8000";

/// A configuration directory of its own, removed when dropped.
struct Directory(PathBuf);

impl Directory {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("gluetun-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        let _ = std::fs::create_dir_all(&path);
        Self(path)
    }

    fn roles(&self) -> String {
        std::fs::read_to_string(self.0.join(ROLES)).unwrap_or_default()
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn a_key() -> String {
    ('a'..='p').collect()
}

fn another_key() -> String {
    ('q'..='z').collect()
}

fn gluetun(http: &Arc<Fake>, directory: &Directory) -> Gluetun {
    Gluetun::new(http.clone(), BASE, Arc::new(a_key()), &directory.0)
}

fn replying(answers: Vec<(u16, &'static str)>) -> Arc<Fake> {
    Fake::in_turn(
        answers
            .into_iter()
            .map(|(status, body)| Answer::reply(status, body))
            .collect(),
    )
}

struct Nothing;

impl Holding for Nothing {
    fn key(&self) -> Result<String, String> {
        Err("upstream.json could not be read: not found".to_owned())
    }
}

#[tokio::test]
async fn a_running_tunnel_is_up_and_asked_under_the_key() {
    let directory = Directory::new("up");
    let http = replying(vec![(200, r#"{"status":"running"}"#)]);
    assert_eq!(gluetun(&http, &directory).up().await.ok(), Some(true));
    let asked = http.requests();
    assert!(asked.iter().all(|one| one.method == Method::Get
        && one.url == format!("{BASE}/v1/vpn/status")
        && one.headers == [("X-API-Key".to_owned(), a_key())]));
}

#[tokio::test]
async fn a_tunnel_in_any_other_state_is_not_up() {
    let directory = Directory::new("down");
    for status in ["stopped", "starting", "crashed"] {
        let http = Fake::always(Answer::reply(200, format!(r#"{{"status":"{status}"}}"#)));
        assert_eq!(gluetun(&http, &directory).up().await.ok(), Some(false));
    }
}

#[tokio::test]
async fn a_refused_key_a_silent_server_and_an_unreadable_answer_are_each_said() {
    let directory = Directory::new("failures");
    let refused = gluetun(&replying(vec![(401, "Unauthorized")]), &directory)
        .up()
        .await;
    assert!(
        matches!(refused, Err(Failure::Unauthorised { .. })),
        "{refused:?}"
    );
    let silent = gluetun(&Fake::silent(), &directory).up().await;
    assert!(
        matches!(silent, Err(Failure::Unavailable { .. })),
        "{silent:?}"
    );
    let unreadable = gluetun(&replying(vec![(200, "running")]), &directory)
        .forwarded()
        .await;
    assert!(
        matches!(&unreadable, Err(Failure::Refused { detail, .. }) if detail.contains("forwarded ports")),
        "{unreadable:?}"
    );
}

#[tokio::test]
async fn without_a_key_held_nothing_is_asked_or_written() {
    let directory = Directory::new("unkeyed");
    let http = Fake::silent();
    let unkeyed = Gluetun::new(http.clone(), BASE, Arc::new(Nothing), &directory.0);
    for answer in [
        unkeyed.up().await.map(|_| ()),
        unkeyed.forwarded().await.map(|_| ()),
        unkeyed.secure().await.map(|_| ()),
    ] {
        assert!(
            matches!(&answer, Err(Failure::Refused { detail, .. }) if detail.contains("holds no key")),
            "{answer:?}"
        );
    }
    assert!(http.requests().is_empty());
    assert!(!directory.0.join(ROLES).exists());
}

#[tokio::test]
async fn the_first_port_forwarded_is_answered_and_none_where_nothing_is() {
    let directory = Directory::new("forwarded");
    let http = replying(vec![
        (200, r#"{"port":5914,"ports":[5914,5915]}"#),
        (200, r#"{"port":0,"ports":null}"#),
        (200, r#"{"port":0,"ports":[0]}"#),
        (200, "{}"),
    ]);
    let gluetun = gluetun(&http, &directory);
    let mut answers = Vec::new();
    for _ in 0..4 {
        answers.push(gluetun.forwarded().await.ok());
    }
    assert_eq!(
        answers,
        [Some(Some(5914)), Some(None), Some(None), Some(None)]
    );
    assert!(http
        .requests()
        .iter()
        .all(|one| one.url == format!("{BASE}/v1/portforward")
            && one.headers == [("X-API-Key".to_owned(), a_key())]));
}

#[tokio::test]
async fn securing_writes_one_role_opening_the_two_routes_to_the_key_alone() {
    let directory = Directory::new("written");
    let http = Fake::silent();
    let answer = gluetun(&http, &directory).secure().await;
    assert_eq!(answer.ok(), Some(Applied::OnRestart));
    assert!(http.requests().is_empty());
    let read: Result<toml::Table, _> = directory.roles().parse();
    let expected: Result<toml::Table, _> = format!(
        "[[roles]]\nname = \"lemonfiber\"\nroutes = [\"GET /v1/vpn/status\", \"GET /v1/portforward\"]\nauth = \"apikey\"\napikey = \"{}\"\n",
        a_key()
    )
    .parse();
    assert_eq!(read.ok(), expected.ok());
    let mode = std::fs::metadata(directory.0.join(ROLES)).map(|held| held.permissions().mode());
    assert_eq!(mode.ok().map(|mode| mode & 0o777), Some(0o600));
}

#[tokio::test]
async fn a_role_file_gluetun_holds_is_answered_as_held_now() {
    let directory = Directory::new("held");
    let written = gluetun(&Fake::silent(), &directory).secure().await;
    assert_eq!(written.ok(), Some(Applied::OnRestart));
    let http = replying(vec![
        (200, r#"{"status":"running"}"#),
        (401, "Unauthorized"),
    ]);
    assert_eq!(
        gluetun(&http, &directory).secure().await.ok(),
        Some(Applied::Now)
    );
    let asked = http.requests();
    assert_eq!(
        asked
            .iter()
            .map(|one| one.headers.len())
            .collect::<Vec<_>>(),
        [1, 0]
    );
}

#[tokio::test]
async fn a_role_file_gluetun_has_not_read_yet_is_taken_up_on_restart() {
    let directory = Directory::new("unread");
    let _ = gluetun(&Fake::silent(), &directory).secure().await;
    let before = directory.roles();
    for answers in [
        vec![
            (200, r#"{"status":"running"}"#),
            (200, r#"{"status":"running"}"#),
        ],
        vec![(401, "Unauthorized"), (401, "Unauthorized")],
    ] {
        assert_eq!(
            gluetun(&replying(answers), &directory).secure().await.ok(),
            Some(Applied::OnRestart)
        );
    }
    assert_eq!(
        gluetun(&Fake::silent(), &directory).secure().await.ok(),
        Some(Applied::OnRestart)
    );
    assert_eq!(directory.roles(), before);
}

#[tokio::test]
async fn a_role_file_for_another_key_is_written_over() {
    let directory = Directory::new("rotated");
    let _ = Gluetun::new(Fake::silent(), BASE, Arc::new(another_key()), &directory.0)
        .secure()
        .await;
    assert!(directory.roles().contains(&another_key()));
    let http = Fake::silent();
    assert_eq!(
        gluetun(&http, &directory).secure().await.ok(),
        Some(Applied::OnRestart)
    );
    assert!(directory.roles().contains(&a_key()));
    assert!(!directory.roles().contains(&another_key()));
    assert!(http.requests().is_empty());
}

#[tokio::test]
async fn a_link_planted_as_the_role_file_is_replaced_and_never_written_through() {
    let directory = Directory::new("planted");
    let elsewhere = Directory::new("planted-elsewhere");
    let target = elsewhere.0.join("target");
    let _ = std::fs::write(&target, "kept");
    let _ = std::os::unix::fs::symlink(&target, directory.0.join(ROLES));
    assert_eq!(
        gluetun(&Fake::silent(), &directory).secure().await.ok(),
        Some(Applied::OnRestart)
    );
    assert_eq!(
        std::fs::read_to_string(&target).ok().as_deref(),
        Some("kept")
    );
    assert!(directory.roles().contains(&a_key()));
    assert!(std::fs::symlink_metadata(directory.0.join(ROLES)).is_ok_and(|held| held.is_file()));
}

#[tokio::test]
async fn a_directory_not_there_leaves_gluetun_unavailable_and_a_linked_one_is_refused() {
    let directory = Directory::new("missing");
    let missing = Gluetun::new(
        Fake::silent(),
        BASE,
        Arc::new(a_key()),
        &directory.0.join("absent"),
    )
    .secure()
    .await;
    assert!(
        matches!(missing, Err(Failure::Unavailable { .. })),
        "{missing:?}"
    );
    let elsewhere = Directory::new("linked-elsewhere");
    let linked = directory.0.join("linked");
    let _ = std::os::unix::fs::symlink(&elsewhere.0, &linked);
    let refused = Gluetun::new(Fake::silent(), BASE, Arc::new(a_key()), &linked)
        .secure()
        .await;
    assert!(
        matches!(refused, Err(Failure::Refused { .. })),
        "{refused:?}"
    );
    assert!(!elsewhere.0.join(ROLES).exists());
}

#[tokio::test]
async fn a_role_file_that_cannot_be_written_leaves_gluetun_unavailable() {
    let directory = Directory::new("unwritable");
    let _ = std::fs::set_permissions(&directory.0, std::fs::Permissions::from_mode(0o500));
    let answer = gluetun(&Fake::silent(), &directory).secure().await;
    let _ = std::fs::set_permissions(&directory.0, std::fs::Permissions::from_mode(0o700));
    assert!(
        matches!(answer, Err(Failure::Unavailable { .. })),
        "{answer:?}"
    );
}

#[tokio::test]
async fn the_control_server_answers_whether_or_not_it_admits_the_request() {
    let directory = Directory::new("answers");
    assert!(
        gluetun(&replying(vec![(401, "Unauthorized")]), &directory)
            .answers()
            .await
    );
    assert!(!gluetun(&Fake::silent(), &directory).answers().await);
}
