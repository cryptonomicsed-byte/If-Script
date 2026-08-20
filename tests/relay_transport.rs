//! End-to-end relay transport tests against a loopback mock relay.
//!
//! The unit tests in `src/nostr/relay.rs` cover frame parsing without a socket.
//! These drive the actual socket path — connect, NIP-42 handshake, publish,
//! read-back — against a minimal relay running on `127.0.0.1`, so the protocol
//! sequencing is exercised without depending on an external relay being
//! reachable, admitting our key, or being up.
//!
//! The mock is deliberately strict about the things that have bitten this
//! ecosystem: it refuses writes before auth, and it answers `REQ` only for
//! events it genuinely stored.

use std::net::TcpListener;
use std::thread;

use ifascript::vm::IfaVM;
use ifascript::{CastReceipt, NostrIdentity};
use ifascript::nostr::relay::{RelayConnection, RelayError};
use serde_json::Value;

/// How the mock should behave, so a test can pick the failure it wants to see.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// Challenge, accept auth, accept and store events.
    Normal,
    /// Challenge, then reject the auth event.
    RejectAuth,
    /// Challenge, accept auth, then reject the published event.
    RejectEvent,
    /// Accept the publish but never serve the event back.
    AcceptButDoNotStore,
}

/// Start a mock relay on an ephemeral port. Returns its `ws://` URL.
fn start_mock_relay(mode: Mode) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        let Ok((stream, _)) = listener.accept() else {
            return;
        };
        let mut ws = match tungstenite::accept(stream) {
            Ok(ws) => ws,
            Err(_) => return,
        };

        let mut authenticated = false;
        let mut stored: Vec<Value> = Vec::new();

        // NIP-42: the relay opens with a challenge.
        let _ = ws.send(tungstenite::Message::Text(
            r#"["AUTH","mock-challenge"]"#.into(),
        ));

        loop {
            let msg = match ws.read() {
                Ok(tungstenite::Message::Text(t)) => t,
                Ok(tungstenite::Message::Close(_)) | Err(_) => break,
                _ => continue,
            };
            let Ok(frame) = serde_json::from_str::<Vec<Value>>(&msg) else {
                continue;
            };
            let verb = frame.first().and_then(Value::as_str).unwrap_or_default();

            match verb {
                "AUTH" => {
                    let id = frame
                        .get(1)
                        .and_then(|e| e.get("id"))
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    if mode == Mode::RejectAuth {
                        let _ = ws.send(tungstenite::Message::Text(
                            format!(r#"["OK","{id}",false,"auth-required: rejected"]"#).into(),
                        ));
                    } else {
                        authenticated = true;
                        let _ = ws.send(tungstenite::Message::Text(
                            format!(r#"["OK","{id}",true,""]"#).into(),
                        ));
                    }
                }
                "EVENT" => {
                    let event = frame.get(1).cloned().unwrap_or(Value::Null);
                    let id = event
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();

                    // The real relay refuses writes before auth, and reports it
                    // as an OK-false that reads like something else.
                    if !authenticated {
                        let _ = ws.send(tungstenite::Message::Text(
                            format!(r#"["OK","{id}",false,"auth-required: not authenticated"]"#)
                                .into(),
                        ));
                        continue;
                    }
                    if mode == Mode::RejectEvent {
                        let _ = ws.send(tungstenite::Message::Text(
                            format!(r#"["OK","{id}",false,"restricted: unknown event kind"]"#)
                                .into(),
                        ));
                        continue;
                    }
                    if mode != Mode::AcceptButDoNotStore {
                        stored.push(event);
                    }
                    let _ = ws.send(tungstenite::Message::Text(
                        format!(r#"["OK","{id}",true,""]"#).into(),
                    ));
                }
                "REQ" => {
                    let sub = frame.get(1).and_then(Value::as_str).unwrap_or_default();
                    let wanted: Vec<String> = frame
                        .get(2)
                        .and_then(|f| f.get("ids"))
                        .and_then(Value::as_array)
                        .map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();

                    for ev in &stored {
                        let id = ev.get("id").and_then(Value::as_str).unwrap_or_default();
                        if wanted.iter().any(|w| w == id) {
                            let _ = ws.send(tungstenite::Message::Text(
                                format!(r#"["EVENT","{sub}",{ev}]"#).into(),
                            ));
                        }
                    }
                    let _ = ws.send(tungstenite::Message::Text(
                        format!(r#"["EOSE","{sub}"]"#).into(),
                    ));
                }
                "CLOSE" => {}
                _ => {}
            }
        }
    });

    // No readiness probe: `bind` already put the socket in the listening state
    // on this thread, so a client connect queues in the backlog. Probing with a
    // throwaway TcpStream would be worse than useless here -- it consumes the
    // mock's single `accept()`, and the real client then cannot connect.
    format!("ws://127.0.0.1:{port}")
}

fn an_engram(identity: &NostrIdentity) -> nostr::Event {
    let cast = IfaVM::new().cast_odu();
    let receipt = CastReceipt::from_cast(&cast, true).with_receipt_hash("relay-test");
    ifascript::nostr::cast_engram(identity, &receipt, identity.public_key_hex()).unwrap()
}

#[test]
fn the_full_authenticated_publish_path_works() {
    let url = start_mock_relay(Mode::Normal);
    let identity = NostrIdentity::generate();

    let mut conn = RelayConnection::connect_authenticated(&url, &identity)
        .expect("connect and authenticate");
    assert!(conn.is_authenticated());

    let event = an_engram(&identity);
    conn.publish(&event).expect("publish should be accepted");
    conn.close();
}

#[test]
fn a_verified_publish_reads_the_event_back() {
    let url = start_mock_relay(Mode::Normal);
    let identity = NostrIdentity::generate();

    let mut conn = RelayConnection::connect_authenticated(&url, &identity).unwrap();
    let event = an_engram(&identity);

    // The whole point: not just an OK, but the relay serving it back.
    conn.publish_verified(&event)
        .expect("event should be retrievable after publish");
    conn.close();
}

#[test]
fn an_accepted_but_unstored_event_is_reported_as_not_retrievable() {
    // The failure mode `publish_verified` exists for: the relay says OK and the
    // data is not there. A plain `publish` cannot tell the difference.
    let url = start_mock_relay(Mode::AcceptButDoNotStore);
    let identity = NostrIdentity::generate();

    let mut conn = RelayConnection::connect_authenticated(&url, &identity).unwrap();
    let event = an_engram(&identity);

    conn.publish(&event).expect("relay claims to accept it");

    let url2 = start_mock_relay(Mode::AcceptButDoNotStore);
    let mut conn2 = RelayConnection::connect_authenticated(&url2, &identity).unwrap();
    let err = conn2.publish_verified(&event).unwrap_err();
    assert!(
        matches!(err, RelayError::NotRetrievable(_)),
        "expected NotRetrievable, got {err:?}"
    );
}

#[test]
fn a_rejected_event_is_an_error_never_a_success() {
    // minipae's bug: this exact shape reported as success.
    let url = start_mock_relay(Mode::RejectEvent);
    let identity = NostrIdentity::generate();

    let mut conn = RelayConnection::connect_authenticated(&url, &identity).unwrap();
    let event = an_engram(&identity);

    let err = conn.publish(&event).unwrap_err();
    match err {
        RelayError::Rejected { message, .. } => {
            assert!(message.contains("unknown event kind"), "message: {message}");
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[test]
fn publishing_before_auth_surfaces_the_relays_refusal() {
    // Connect without authenticating: the relay refuses the write, and that
    // refusal must reach the caller rather than being swallowed.
    let url = start_mock_relay(Mode::Normal);
    let identity = NostrIdentity::generate();

    let mut conn = RelayConnection::connect(&url).unwrap();
    assert!(!conn.is_authenticated());

    let event = an_engram(&identity);
    let err = conn.publish(&event).unwrap_err();
    match err {
        RelayError::Rejected { message, .. } => {
            assert!(message.contains("auth-required"), "message: {message}");
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[test]
fn a_refused_auth_fails_at_connect_time() {
    let url = start_mock_relay(Mode::RejectAuth);
    let identity = NostrIdentity::generate();

    let err = RelayConnection::connect_authenticated(&url, &identity).unwrap_err();
    assert!(
        matches!(err, RelayError::AuthFailed(_)),
        "expected AuthFailed, got {err:?}"
    );
}

#[test]
fn connecting_to_a_dead_address_errors_rather_than_hanging() {
    let identity = NostrIdentity::generate();
    // Port 1 on loopback: nothing listens, connection refused immediately.
    let err = RelayConnection::connect_authenticated("ws://127.0.0.1:1", &identity).unwrap_err();
    assert!(matches!(err, RelayError::Connect { .. }), "got {err:?}");
}

#[test]
fn the_gateway_publishes_to_every_registered_relay() {
    use ifascript::NostrGateway;

    let a = start_mock_relay(Mode::Normal);
    let b = start_mock_relay(Mode::Normal);

    let identity = NostrIdentity::generate();
    let event = an_engram(&identity);

    let mut gateway = NostrGateway::new(identity);
    gateway.add_relay(a.clone());
    gateway.add_relay(b.clone());

    let (ok, failed) = gateway.publish_everywhere(&event);
    assert_eq!(ok.len(), 2, "both relays should accept; failures: {failed:?}");
    assert!(failed.is_empty());
    // Both relays are now recorded as authenticated.
    assert!(gateway.relays().iter().all(|r| r.is_authenticated()));
}

#[test]
fn one_relay_refusing_does_not_hide_another_succeeding() {
    // Partial success is the normal case with multiple relays. Reporting it as
    // total failure would be as wrong as reporting it as total success.
    use ifascript::NostrGateway;

    let good = start_mock_relay(Mode::Normal);
    let bad = start_mock_relay(Mode::RejectEvent);

    let identity = NostrIdentity::generate();
    let event = an_engram(&identity);

    let mut gateway = NostrGateway::new(identity);
    gateway.add_relay(good.clone());
    gateway.add_relay(bad.clone());

    let (ok, failed) = gateway.publish_everywhere(&event);
    assert_eq!(ok, vec![good]);
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].0, bad);
}
