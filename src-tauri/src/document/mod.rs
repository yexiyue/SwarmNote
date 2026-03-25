pub mod commands;

use crate::error::{AppError, AppResult};
use crate::identity::IdentityState;

pub(crate) fn peer_id(identity: &IdentityState) -> AppResult<String> {
    let info = identity
        .device_info
        .read()
        .map_err(|e| AppError::Identity(format!("lock error: {e}")))?;
    Ok(info.peer_id.clone())
}
