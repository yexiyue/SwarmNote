//! 设备配对：配对码生成、DHT 发现、请求/响应、已配对设备管理。

pub mod code;
pub mod commands;
pub mod manager;

pub use manager::PairingManager;
