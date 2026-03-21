use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "paired_devices")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub peer_id: String,
    pub public_key: Vec<u8>,
    pub device_name: String,
    pub os_info: Option<String>,
    pub paired_at: i64,
}

impl ActiveModelBehavior for ActiveModel {}
