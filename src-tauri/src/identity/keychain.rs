use keyring::Entry;
use log::{info, warn};
use swarm_p2p_core::libp2p::identity::Keypair;

const SERVICE: &str = "com.swarmnote";
const KEYPAIR_KEY: &str = "ed25519-keypair";

/// 从系统钥匙串加载已有的 Ed25519 密钥对，
/// 若不存在则生成新密钥对并存储。
pub fn load_or_generate_keypair() -> Result<Keypair, crate::identity::IdentityError> {
    match load_keypair_from_keychain() {
        Ok(Some(keypair)) => {
            info!("Loaded existing keypair from system keychain");
            Ok(keypair)
        }
        Ok(None) => {
            info!("No existing keypair found, generating new Ed25519 keypair");
            let keypair = Keypair::generate_ed25519();
            save_keypair_to_keychain(&keypair)?;
            info!("New keypair saved to system keychain");
            Ok(keypair)
        }
        Err(e) => {
            warn!("System keychain unavailable: {e}, generating ephemeral keypair");
            Ok(Keypair::generate_ed25519())
        }
    }
}

fn load_keypair_from_keychain() -> Result<Option<Keypair>, crate::identity::IdentityError> {
    let entry = Entry::new(SERVICE, KEYPAIR_KEY)
        .map_err(|e| crate::identity::IdentityError::Keychain(e.to_string()))?;

    match entry.get_secret() {
        Ok(bytes) => {
            let keypair = Keypair::from_protobuf_encoding(&bytes)
                .map_err(|e| crate::identity::IdentityError::KeypairDecode(e.to_string()))?;
            Ok(Some(keypair))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(crate::identity::IdentityError::Keychain(e.to_string())),
    }
}

fn save_keypair_to_keychain(keypair: &Keypair) -> Result<(), crate::identity::IdentityError> {
    let bytes = keypair
        .to_protobuf_encoding()
        .map_err(|e| crate::identity::IdentityError::KeypairEncode(e.to_string()))?;

    let entry = Entry::new(SERVICE, KEYPAIR_KEY)
        .map_err(|e| crate::identity::IdentityError::Keychain(e.to_string()))?;

    entry
        .set_secret(&bytes)
        .map_err(|e| crate::identity::IdentityError::Keychain(e.to_string()))?;

    Ok(())
}
