//! SWIM cluster membership with Lifeguard-style suspicion.

pub mod member;
pub mod network;
pub mod swim;

pub use member::{Member, MemberState};
pub use network::{decode_message, encode_message, GossipMesh, GossipNetError};
pub use swim::{GossipMessage, SwimNode};
