//! `MessageRead` — per-recipient read acknowledgment for any `Message`.
//!
//! Replaces the old single-bool `Message.read_at` so broadcast messages
//! (one row addressed to a room, fanned out to N members) can track
//! per-member ack without ambiguity. A message with zero `MessageRead`
//! rows is unread by everyone; a 1:1 message gets at most one row; a
//! broadcast can have up to N (one per room member who's caught up).
//!
//! Composite id is `"<message_id>::<session_id>"`, so re-acking is
//! idempotent.

use myko::myko_item;
use myko::prelude::{EventPublishing as _, Querying as _, RegistryScoped as _, RequestScoped as _};

use crate::message::Message;

#[myko_item]
pub struct MessageRead {
    /// Cascade on the *message*: an ack is meaningless once its message is gone,
    /// so it dies with the message. This is the ack's primary lifetime owner.
    #[belongs_to(Message)]
    pub message_id: crate::message::MessageId,

    /// NOT a `belongs_to(Session)` cascade: read-state is sticky. A session's
    /// id is stable across a pi exit / `/reload` (sha1 of the session file), so
    /// a session DEL is a *temporary* disconnect, not a permanent one — and the
    /// reader will come back. If the ack cascaded on session DEL, every reload
    /// re-unacked the whole backlog and the pull-inbox re-injected it (the
    /// "20-message dump of DMs I already saw" report). The ack persists as a
    /// durable fact about (message, reader) for as long as the message exists.
    pub session_id: crate::session::SessionId,

    /// Wall-clock millis when this session marked the message read.
    pub read_at: i64,
}

impl MessageRead {
    /// Compute the composite id for a (message, session) pair.
    pub fn make_id(message_id: &str, session_id: &str) -> String {
        format!("{message_id}::{session_id}")
    }
}
