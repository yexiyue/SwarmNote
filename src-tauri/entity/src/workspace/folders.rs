use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "folders")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub workspace_id: String,
    pub parent_folder_id: Option<String>,
    pub name: String,
    pub rel_path: String,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[sea_orm(belongs_to, from = "workspace_id", to = "id")]
    pub workspace: HasOne<super::workspaces::Entity>,
    #[sea_orm(
        self_ref,
        relation_enum = "ParentFolder",
        relation_reverse = "ChildFolders",
        from = "parent_folder_id",
        to = "id"
    )]
    pub parent_folder: HasOne<Entity>,
    #[sea_orm(
        self_ref,
        relation_enum = "ChildFolders",
        relation_reverse = "ParentFolder"
    )]
    pub child_folders: HasMany<Entity>,
    #[sea_orm(has_many)]
    pub documents: HasMany<super::documents::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
