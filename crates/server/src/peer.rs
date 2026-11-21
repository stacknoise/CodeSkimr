//! The address of the program that is connected, as the accept loop saw it.

use std::net::SocketAddr;

/// The peer address of the connection a request came in on. The accept loop puts it into the
/// request; a client cannot set it, because it is not part of the request text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerAddr(pub SocketAddr);

impl PeerAddr {
    /// Whether the peer is on this machine. The server listens on the loopback address only, so
    /// this is a second check for the route that opens a window, not the first.
    pub fn is_local(self) -> bool {
        self.0.ip().is_loopback()
    }
}
