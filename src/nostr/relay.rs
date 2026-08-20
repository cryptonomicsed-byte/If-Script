// ifascript/src/nostr/relay.rs
//! Relay transport — actually putting a signed event on the wire.
//!
//! Everything else in this module tree prepares and signs. This is the part
//! that opens a socket, authenticates, publishes, and reads back.
//!
//! # Why the frame parsing is separated from the socket
//!
//! minipae shipped a real bug here: `publish()` misread NIP-01's `OK` frame and
//! reported the relay's `auth-required: not authenticated` **rejection** as
//! `ok: True`. The publish looked successful and the data was never stored.
//!
//! That bug is not a networking bug — it is a parsing bug, and parsing is
//! testable without a relay. So [`RelayMessage`] and [`parse_ok`] are pure
//! functions over frame text, exercised directly by tests, including a test
//! built from the exact rejection frame that fooled minipae. The socket code
//! above them stays thin enough to have little room for its own logic.
//!
//! # Why a write is not believed until it is read back
//!
//! An `OK` frame is the relay saying it accepted the event, which is not the
//! same as the event being retrievable. [`RelayConnection::publish_verified`]
//! therefore republishes nothing and asserts nothing on the strength of `OK`
//! alone: it issues a separate `REQ` for the event id and confirms the relay
//! serves it back. Slower, and the only version worth trusting.

use std::net::TcpStream;
use std::time::Duration;

use nostr::{Event, EventId};
use serde_json::Value;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

use super::identity::NostrIdentity;
use super::{events, EventError};

/// Errors from talking to a relay.
#[derive(Debug, thiserror::Error)]
pub enum RelayError {
    #[error("connecting to {url} failed: {source}")]
    Connect {
        url: String,
        #[source]
        source: Box<tungstenite::Error>,
    },
    #[error("websocket transport error: {0}")]
    Transport(String),
    #[error("relay sent a frame that is not valid JSON: {0}")]
    MalformedFrame(String),
    /// The relay explicitly refused the event. `message` is its stated reason —
    /// `restricted: unknown event kind`, `auth-required: …`, and so on.
    #[error("relay rejected event {event_id}: {message}")]
    Rejected { event_id: String, message: String },
    #[error("NIP-42 authentication failed: {0}")]
    AuthFailed(String),
    /// The relay accepted the event but does not serve it back.
    #[error("event {0} was accepted but could not be read back")]
    NotRetrievable(String),
    #[error("timed out waiting for {expected}")]
    Timeout { expected: String },
    #[error(transparent)]
    Event(#[from] EventError),
}

/// A NIP-01 message from a relay, in the forms this client acts on.
#[derive(Debug, Clone, PartialEq)]
pub enum RelayMessage {
    /// `["OK", <event_id>, <accepted>, <message>]`
    Ok {
        event_id: String,
        accepted: bool,
        message: String,
    },
    /// `["AUTH", <challenge>]` — NIP-42.
    Auth { challenge: String },
    /// `["EVENT", <sub_id>, <event>]`
    Event {
        sub_id: String,
        event: Box<Value>,
    },
    /// `["EOSE", <sub_id>]` — end of stored events.
    EndOfStoredEvents { sub_id: String },
    /// `["CLOSED", <sub_id>, <message>]`
    Closed { sub_id: String, message: String },
    /// `["NOTICE", <message>]`
    Notice { message: String },
    /// A well-formed JSON array this client has no handling for.
    Unhandled(String),
}

/// Parse one relay frame.
///
/// Deliberately total over well-formed JSON: an unrecognised verb becomes
/// [`RelayMessage::Unhandled`] rather than an error, so a relay extension
/// cannot break a publish that was otherwise fine.
pub fn parse_frame(text: &str) -> Result<RelayMessage, RelayError> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| RelayError::MalformedFrame(e.to_string()))?;
    let arr = value
        .as_array()
        .ok_or_else(|| RelayMessage::Unhandled(text.to_string()))
        .map_err(|_| RelayError::MalformedFrame("frame is not a JSON array".into()))?;

    let verb = arr.first().and_then(Value::as_str).unwrap_or_default();

    Ok(match verb {
        "OK" => parse_ok(arr)?,
        "AUTH" => RelayMessage::Auth {
            challenge: arr.get(1).and_then(Value::as_str).unwrap_or_default().to_string(),
        },
        "EVENT" => RelayMessage::Event {
            sub_id: arr.get(1).and_then(Value::as_str).unwrap_or_default().to_string(),
            event: Box::new(arr.get(2).cloned().unwrap_or(Value::Null)),
        },
        "EOSE" => RelayMessage::EndOfStoredEvents {
            sub_id: arr.get(1).and_then(Value::as_str).unwrap_or_default().to_string(),
        },
        "CLOSED" => RelayMessage::Closed {
            sub_id: arr.get(1).and_then(Value::as_str).unwrap_or_default().to_string(),
            message: arr.get(2).and_then(Value::as_str).unwrap_or_default().to_string(),
        },
        "NOTICE" => RelayMessage::Notice {
            message: arr.get(1).and_then(Value::as_str).unwrap_or_default().to_string(),
        },
        _ => RelayMessage::Unhandled(text.to_string()),
    })
}

/// Parse an `OK` frame: `["OK", <event_id>, <accepted:bool>, <message>]`.
///
/// The acceptance flag is **index 2**. Index 1 is the event id, which is always
/// a non-empty string and therefore always truthy — reading it as the flag is
/// exactly how minipae came to report rejections as successes. A frame whose
/// index 2 is not a real boolean is treated as **not accepted**: a relay that
/// does not clearly say yes has not said yes.
fn parse_ok(arr: &[Value]) -> Result<RelayMessage, RelayError> {
    let event_id = arr.get(1).and_then(Value::as_str).unwrap_or_default().to_string();
    let accepted = arr.get(2).and_then(Value::as_bool).unwrap_or(false);
    let message = arr.get(3).and_then(Value::as_str).unwrap_or_default().to_string();
    Ok(RelayMessage::Ok {
        event_id,
        accepted,
        message,
    })
}

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

/// An open, optionally authenticated connection to one relay.
///
/// `Debug` reports the relay and auth state only — the socket has no useful
/// rendering, and this type is held alongside signing identities.
pub struct RelayConnection {
    socket: Socket,
    url: String,
    authenticated: bool,
}

impl std::fmt::Debug for RelayConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelayConnection")
            .field("url", &self.url)
            .field("authenticated", &self.authenticated)
            .finish()
    }
}

impl RelayConnection {
    /// Open a connection. Does not authenticate — call [`Self::authenticate`],
    /// or use [`Self::connect_authenticated`].
    pub fn connect(url: &str) -> Result<Self, RelayError> {
        let (socket, _response) = connect(url).map_err(|e| RelayError::Connect {
            url: url.to_string(),
            source: Box::new(e),
        })?;

        let mut conn = Self {
            socket,
            url: url.to_string(),
            authenticated: false,
        };
        conn.set_read_timeout(Duration::from_secs(30))?;
        Ok(conn)
    }

    /// Open a connection and complete NIP-42 authentication.
    ///
    /// The Buzz relay refuses writes from unauthenticated connections, and
    /// reports that refusal at publish time as a rejection that reads like an
    /// unrelated failure. Authenticating up front turns that into one clear
    /// error at connect time.
    pub fn connect_authenticated(
        url: &str,
        identity: &NostrIdentity,
    ) -> Result<Self, RelayError> {
        let mut conn = Self::connect(url)?;
        conn.authenticate(identity)?;
        Ok(conn)
    }

    fn set_read_timeout(&mut self, timeout: Duration) -> Result<(), RelayError> {
        let stream = match self.socket.get_mut() {
            MaybeTlsStream::Plain(s) => s,
            MaybeTlsStream::NativeTls(s) => s.get_mut(),
            _ => return Ok(()),
        };
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| RelayError::Transport(e.to_string()))
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Complete the NIP-42 handshake: wait for the relay's `AUTH` challenge,
    /// answer with a signed `kind:22242`, and require an accepting `OK`.
    pub fn authenticate(&mut self, identity: &NostrIdentity) -> Result<(), RelayError> {
        let challenge = loop {
            match self.read_message()? {
                RelayMessage::Auth { challenge } => break challenge,
                // A relay may greet with a NOTICE first; keep waiting.
                RelayMessage::Notice { .. } | RelayMessage::Unhandled(_) => continue,
                other => {
                    return Err(RelayError::AuthFailed(format!(
                        "expected an AUTH challenge, got {other:?}"
                    )))
                }
            }
        };

        let auth_event = events::relay_auth(identity, &challenge, &self.url)?;
        self.send(&format!(
            "[\"AUTH\",{}]",
            serde_json::to_string(&auth_event)
                .map_err(|e| RelayError::Transport(e.to_string()))?
        ))?;

        loop {
            match self.read_message()? {
                RelayMessage::Ok {
                    accepted, message, ..
                } => {
                    if !accepted {
                        return Err(RelayError::AuthFailed(message));
                    }
                    self.authenticated = true;
                    return Ok(());
                }
                RelayMessage::Notice { .. } | RelayMessage::Unhandled(_) => continue,
                other => {
                    return Err(RelayError::AuthFailed(format!(
                        "expected OK for the auth event, got {other:?}"
                    )))
                }
            }
        }
    }

    /// Publish a signed event and require an accepting `OK`.
    ///
    /// Returns the relay's message on success (often empty). A rejection —
    /// including `auth-required` and `restricted: unknown event kind` — is a
    /// [`RelayError::Rejected`], never a success.
    ///
    /// This confirms the relay *said* yes. For a write you intend to rely on,
    /// use [`Self::publish_verified`].
    pub fn publish(&mut self, event: &Event) -> Result<String, RelayError> {
        let payload = serde_json::to_string(event)
            .map_err(|e| RelayError::Transport(e.to_string()))?;
        self.send(&format!("[\"EVENT\",{payload}]"))?;

        let want = event.id().to_hex();
        loop {
            match self.read_message()? {
                RelayMessage::Ok {
                    event_id,
                    accepted,
                    message,
                } => {
                    // A relay may be answering about a different event; only
                    // this one's verdict counts.
                    if event_id != want {
                        continue;
                    }
                    if !accepted {
                        return Err(RelayError::Rejected { event_id, message });
                    }
                    return Ok(message);
                }
                RelayMessage::Notice { .. } | RelayMessage::Unhandled(_) => continue,
                RelayMessage::Closed { message, .. } => {
                    return Err(RelayError::Rejected {
                        event_id: want,
                        message,
                    })
                }
                _ => continue,
            }
        }
    }

    /// Publish, then independently read the event back before reporting success.
    ///
    /// An `OK` is the relay's assertion, not proof of storage. This issues a
    /// separate `REQ` for the id and requires the relay to serve the event.
    pub fn publish_verified(&mut self, event: &Event) -> Result<(), RelayError> {
        self.publish(event)?;
        let id = event.id();
        if !self.has_event(id)? {
            return Err(RelayError::NotRetrievable(id.to_hex()));
        }
        Ok(())
    }

    /// Ask the relay whether it serves an event by id.
    pub fn has_event(&mut self, id: EventId) -> Result<bool, RelayError> {
        let sub = "ifa-verify";
        let filter = format!("{{\"ids\":[\"{}\"],\"limit\":1}}", id.to_hex());
        self.send(&format!("[\"REQ\",\"{sub}\",{filter}]"))?;

        let mut found = false;
        loop {
            match self.read_message()? {
                RelayMessage::Event { sub_id, event } if sub_id == sub => {
                    if event.get("id").and_then(Value::as_str) == Some(&id.to_hex()) {
                        found = true;
                    }
                }
                RelayMessage::EndOfStoredEvents { sub_id } if sub_id == sub => break,
                RelayMessage::Closed { sub_id, message } if sub_id == sub => {
                    return Err(RelayError::Transport(message));
                }
                _ => continue,
            }
        }

        let _ = self.send(&format!("[\"CLOSE\",\"{sub}\"]"));
        Ok(found)
    }

    fn send(&mut self, text: &str) -> Result<(), RelayError> {
        self.socket
            .send(Message::Text(text.into()))
            .map_err(|e| RelayError::Transport(e.to_string()))
    }

    fn read_message(&mut self) -> Result<RelayMessage, RelayError> {
        loop {
            let msg = self
                .socket
                .read()
                .map_err(|e| RelayError::Transport(e.to_string()))?;
            match msg {
                Message::Text(t) => return parse_frame(&t),
                Message::Ping(_) | Message::Pong(_) => continue,
                Message::Close(_) => {
                    return Err(RelayError::Transport("relay closed the connection".into()))
                }
                _ => continue,
            }
        }
    }

    /// Close the connection politely.
    pub fn close(mut self) {
        let _ = self.socket.close(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Frame parsing: the part minipae got wrong, tested without a relay ===

    #[test]
    fn an_accepting_ok_frame_is_read_as_accepted() {
        let f = r#"["OK","abc123",true,""]"#;
        assert_eq!(
            parse_frame(f).unwrap(),
            RelayMessage::Ok {
                event_id: "abc123".into(),
                accepted: true,
                message: String::new()
            }
        );
    }

    #[test]
    fn the_exact_rejection_frame_that_fooled_minipae_is_read_as_a_rejection() {
        // minipae's publish() reported this as ok: True, because index 1 (the
        // event id) is always a truthy string. The acceptance flag is index 2.
        let f = r#"["OK","abc123",false,"auth-required: not authenticated"]"#;
        let RelayMessage::Ok {
            accepted, message, ..
        } = parse_frame(f).unwrap()
        else {
            panic!("expected an OK frame");
        };
        assert!(!accepted, "a rejection must never read as accepted");
        assert_eq!(message, "auth-required: not authenticated");
    }

    #[test]
    fn the_unknown_kind_rejection_is_read_as_a_rejection() {
        // The other rejection this ecosystem actually hits, and the one that
        // arrives *after* a successful auth so it reads like an auth problem.
        let f = r#"["OK","abc",false,"restricted: unknown event kind"]"#;
        let RelayMessage::Ok { accepted, .. } = parse_frame(f).unwrap() else {
            panic!("expected OK");
        };
        assert!(!accepted);
    }

    #[test]
    fn a_non_boolean_acceptance_flag_is_treated_as_rejection() {
        // A relay that does not clearly say yes has not said yes. Defaulting
        // the other way is how a truthy non-bool becomes a false success.
        for f in [
            r#"["OK","abc","true",""]"#,
            r#"["OK","abc",1,""]"#,
            r#"["OK","abc",null,""]"#,
            r#"["OK","abc"]"#,
        ] {
            let RelayMessage::Ok { accepted, .. } = parse_frame(f).unwrap() else {
                panic!("expected OK for {f}");
            };
            assert!(!accepted, "frame {f} must not read as accepted");
        }
    }

    #[test]
    fn an_auth_challenge_is_parsed() {
        assert_eq!(
            parse_frame(r#"["AUTH","challenge-xyz"]"#).unwrap(),
            RelayMessage::Auth {
                challenge: "challenge-xyz".into()
            }
        );
    }

    #[test]
    fn eose_and_closed_and_notice_are_parsed() {
        assert_eq!(
            parse_frame(r#"["EOSE","sub1"]"#).unwrap(),
            RelayMessage::EndOfStoredEvents { sub_id: "sub1".into() }
        );
        assert_eq!(
            parse_frame(r#"["CLOSED","sub1","restricted"]"#).unwrap(),
            RelayMessage::Closed {
                sub_id: "sub1".into(),
                message: "restricted".into()
            }
        );
        assert_eq!(
            parse_frame(r#"["NOTICE","hello"]"#).unwrap(),
            RelayMessage::Notice { message: "hello".into() }
        );
    }

    #[test]
    fn an_event_frame_carries_its_subscription_and_payload() {
        let f = r#"["EVENT","sub1",{"id":"abc","kind":30174}]"#;
        let RelayMessage::Event { sub_id, event } = parse_frame(f).unwrap() else {
            panic!("expected EVENT");
        };
        assert_eq!(sub_id, "sub1");
        assert_eq!(event.get("id").unwrap().as_str().unwrap(), "abc");
    }

    #[test]
    fn an_unrecognised_verb_does_not_error() {
        // A relay extension must not break an otherwise-fine publish.
        assert!(matches!(
            parse_frame(r#"["SOMETHING_NEW","x"]"#).unwrap(),
            RelayMessage::Unhandled(_)
        ));
    }

    #[test]
    fn malformed_json_is_an_error_not_a_silent_success() {
        assert!(matches!(
            parse_frame("not json"),
            Err(RelayError::MalformedFrame(_))
        ));
        assert!(matches!(
            parse_frame(r#"{"not":"an array"}"#),
            Err(RelayError::MalformedFrame(_))
        ));
    }
}
