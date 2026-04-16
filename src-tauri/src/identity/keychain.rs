//! Legacy keypair-loading entry point. Delegates to
//! [`crate::platform::DesktopKeychain`] so there's exactly one path to the
//! OS keychain — previously two independent call sites could each generate
//! their own ephemeral keypair on Linux when Secret Service was unavailable,
//! yielding split-brain PeerIds. This shim disappears in PR #3 once the
//! legacy `IdentityState` is removed.

use swarm_p2p_core::libp2p::identity::Keypair;
use swarmnote_core::KeychainProvider;

use crate::identity::IdentityError;
use crate::platform::DesktopKeychain;

pub fn load_or_generate_keypair() -> Result<Keypair, IdentityError> {
    let bytes = tauri::async_runtime::block_on(DesktopKeychain::new().get_or_create_keypair())
        .map_err(|e| IdentityError::Keychain(e.to_string()))?;
    Keypair::from_protobuf_encoding(&bytes).map_err(|e| IdentityError::KeypairDecode(e.to_string()))
}
