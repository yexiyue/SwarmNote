mod document;
mod folder;
mod workspace;

pub use document::*;
pub use folder::*;
pub use workspace::*;

use crate::identity::IdentityState;

fn peer_id(identity: &IdentityState) -> String {
    identity.device_info.read().unwrap().peer_id.clone()
}
