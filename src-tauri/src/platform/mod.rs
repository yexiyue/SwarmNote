//! Desktop implementations of the `swarmnote-core` platform traits.
//!
//! | Trait              | Impl (this module)     | Backing crate             |
//! |--------------------|------------------------|---------------------------|
//! | `KeychainProvider` | [`keychain::DesktopKeychain`] | `keyring`            |
//! | `EventBus`         | [`event_bus::TauriEventBus`]  | `tauri::AppHandle`   |
//! | `FileWatcher`      | (added in PR #2)       | `notify` + debouncer      |

pub mod event_bus;
pub mod keychain;

pub use event_bus::TauriEventBus;
pub use keychain::DesktopKeychain;
