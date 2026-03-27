use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "paired_devices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub peer_id: String,
    pub hostname: String,
    pub os: Option<String>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub paired_at: i64,
    pub last_seen: Option<i64>,
}

impl ActiveModelBehavior for ActiveModel {}
