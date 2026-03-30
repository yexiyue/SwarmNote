use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub title: String,
    pub rel_path: String,
    pub file_hash: Option<Vec<u8>>,
    pub yjs_state: Option<Vec<u8>>,
    pub state_vector: Option<Vec<u8>>,
    pub lamport_clock: i64,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[sea_orm(belongs_to, from = "workspace_id", to = "id")]
    pub workspace: HasOne<super::workspaces::Entity>,
    #[sea_orm(belongs_to, from = "folder_id", to = "id")]
    pub folder: HasOne<super::folders::Entity>,
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::now_v7()),
            ..ActiveModelTrait::default()
        }
    }
}
