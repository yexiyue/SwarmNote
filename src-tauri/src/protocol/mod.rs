use serde::{Deserialize, Serialize};

// ── OsInfo: 设备信息，通过 libp2p agent_version 交换 ──

/// 设备操作系统信息，嵌入在 agent_version 字符串中通过 Identify 协议自动交换。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OsInfo {
    pub hostname: String,
    pub os: String,
    pub platform: String,
    pub arch: String,
}

impl OsInfo {
    /// 编码为 agent_version 格式：`swarmnote/{version}; os={os}; platform={platform}; arch={arch}; host={hostname}`
    pub fn to_agent_version(&self, version: &str) -> String {
        format!(
            "swarmnote/{}; os={}; platform={}; arch={}; host={}",
            version, self.os, self.platform, self.arch, self.hostname
        )
    }

    /// 从 agent_version 字符串解析 OsInfo。
    /// 返回 None 如果格式不匹配（非 SwarmNote 设备）。
    pub fn from_agent_version(agent_version: &str) -> Option<Self> {
        if !agent_version.starts_with("swarmnote/") {
            return None;
        }

        let mut hostname = String::new();
        let mut os = String::new();
        let mut platform = String::new();
        let mut arch = String::new();

        for part in agent_version.split(';').map(str::trim) {
            if let Some(val) = part.strip_prefix("os=") {
                os = val.to_string();
            } else if let Some(val) = part.strip_prefix("platform=") {
                platform = val.to_string();
            } else if let Some(val) = part.strip_prefix("arch=") {
                arch = val.to_string();
            } else if let Some(val) = part.strip_prefix("host=") {
                hostname = val.to_string();
            }
        }

        Some(OsInfo {
            hostname,
            os,
            platform,
            arch,
        })
    }
}

impl Default for OsInfo {
    fn default() -> Self {
        Self {
            hostname: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_default(),
            os: std::env::consts::OS.to_string(),
            platform: std::env::consts::FAMILY.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}

// ── 顶层协议消息 ──

/// 顶层请求枚举，包装所有子协议。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppRequest {
    Pairing(PairingRequest),
    Sync(SyncRequest),
}

/// 顶层响应枚举，包装所有子协议。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppResponse {
    Pairing(PairingResponse),
    Sync(SyncResponse),
}

// ── 同步子协议 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequest {
    /// 发送本地 state vector，请求对方返回缺失的 updates
    StateVector {
        doc_id: String,
        #[serde(with = "serde_bytes")]
        sv: Vec<u8>,
    },
    /// 请求完整文档状态
    FullSync { doc_id: String },
    /// 查询对方拥有的文档列表
    DocList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    /// 返回请求方缺失的 yjs updates
    Updates {
        doc_id: String,
        #[serde(with = "serde_bytes")]
        updates: Vec<u8>,
    },
    /// 返回文档元数据列表
    DocList { docs: Vec<DocMeta> },
}

/// 文档元数据，用于 DocList 交换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMeta {
    pub doc_id: String,
    pub title: String,
    pub updated_at: i64,
}

// ── 配对子协议 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingRequest {
    pub os_info: OsInfo,
    pub timestamp: i64,
    pub method: PairingMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PairingMethod {
    Code { code: String },
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum PairingResponse {
    Success,
    Refused { reason: PairingRefuseReason },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PairingRefuseReason {
    UserRejected,
    CodeExpired,
    CodeInvalid,
}

// ── 测试 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_info_agent_version_roundtrip() {
        let info = OsInfo {
            hostname: "MacBook-Pro".to_string(),
            os: "macos".to_string(),
            platform: "macos".to_string(),
            arch: "aarch64".to_string(),
        };

        let agent = info.to_agent_version("0.2.0");
        assert_eq!(
            agent,
            "swarmnote/0.2.0; os=macos; platform=macos; arch=aarch64; host=MacBook-Pro"
        );

        let parsed = OsInfo::from_agent_version(&agent).unwrap();
        assert_eq!(parsed, info);
    }

    #[test]
    fn os_info_from_agent_version_non_swarmnote() {
        assert!(OsInfo::from_agent_version("swarmdrop/0.4.0; os=windows").is_none());
        assert!(OsInfo::from_agent_version("some-other-app/1.0").is_none());
    }

    #[test]
    fn os_info_default() {
        let info = OsInfo::default();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn app_request_cbor_roundtrip() {
        let requests = vec![
            AppRequest::Sync(SyncRequest::DocList),
            AppRequest::Sync(SyncRequest::FullSync {
                doc_id: "doc-123".to_string(),
            }),
            AppRequest::Sync(SyncRequest::StateVector {
                doc_id: "doc-456".to_string(),
                sv: vec![1, 2, 3, 4],
            }),
            AppRequest::Pairing(PairingRequest {
                os_info: OsInfo::default(),
                timestamp: 1234567890,
                method: PairingMethod::Code {
                    code: "123456".to_string(),
                },
            }),
            AppRequest::Pairing(PairingRequest {
                os_info: OsInfo::default(),
                timestamp: 1234567890,
                method: PairingMethod::Direct,
            }),
        ];

        for req in requests {
            let json = serde_json::to_string(&req).unwrap();
            let restored: AppRequest = serde_json::from_str(&json).unwrap();
            // Verify it round-trips without panic
            let json2 = serde_json::to_string(&restored).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn app_response_cbor_roundtrip() {
        let responses = vec![
            AppResponse::Sync(SyncResponse::DocList {
                docs: vec![DocMeta {
                    doc_id: "doc-1".to_string(),
                    title: "Test Note".to_string(),
                    updated_at: 1234567890,
                }],
            }),
            AppResponse::Sync(SyncResponse::Updates {
                doc_id: "doc-1".to_string(),
                updates: vec![10, 20, 30],
            }),
            AppResponse::Pairing(PairingResponse::Success),
            AppResponse::Pairing(PairingResponse::Refused {
                reason: PairingRefuseReason::CodeExpired,
            }),
        ];

        for resp in responses {
            let json = serde_json::to_string(&resp).unwrap();
            let restored: AppResponse = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&restored).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn pairing_method_serde_tag() {
        let method = PairingMethod::Code {
            code: "123456".to_string(),
        };
        let json = serde_json::to_string(&method).unwrap();
        assert!(json.contains("\"type\":\"Code\""));
        assert!(json.contains("\"code\":\"123456\""));

        let direct = PairingMethod::Direct;
        let json = serde_json::to_string(&direct).unwrap();
        assert!(json.contains("\"type\":\"Direct\""));
    }
}
