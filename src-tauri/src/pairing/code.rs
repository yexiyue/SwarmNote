use rand::Rng;
use serde::{Deserialize, Serialize};

/// 配对码信息，包含生成的 6 位数字码及其有效期。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingCodeInfo {
    pub code: String,
    pub created_at: i64,
    pub expires_at: i64,
}

impl PairingCodeInfo {
    /// 生成一个 6 位纯数字随机配对码。
    ///
    /// - `expires_in_secs`: 有效期（秒）
    pub fn generate(expires_in_secs: u64) -> Self {
        let code: u32 = rand::rng().random_range(0..1_000_000);
        let code = format!("{code:06}");
        let created_at = chrono::Utc::now().timestamp();
        let expires_at = created_at + expires_in_secs as i64;

        Self {
            code,
            created_at,
            expires_at,
        }
    }

    /// 配对码是否已过期。
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_6_digit_code() {
        let info = PairingCodeInfo::generate(300);
        assert_eq!(info.code.len(), 6);
        assert!(info.code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn generate_sets_expiry_correctly() {
        let info = PairingCodeInfo::generate(600);
        assert_eq!(info.expires_at - info.created_at, 600);
    }

    #[test]
    fn is_expired_returns_false_for_fresh_code() {
        let info = PairingCodeInfo::generate(300);
        assert!(!info.is_expired());
    }

    #[test]
    fn is_expired_returns_true_for_past_code() {
        let info = PairingCodeInfo {
            code: "123456".to_string(),
            created_at: 0,
            expires_at: 1,
        };
        assert!(info.is_expired());
    }
}
