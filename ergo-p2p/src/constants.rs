use std::time::Duration;

/// The timeout for handshakes when connecting to new peers.
#[allow(dead_code)]
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(4);
