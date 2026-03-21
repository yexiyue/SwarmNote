use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "workspaces")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[sea_orm(has_many)]
    pub folders: HasMany<super::folders::Entity>,
    #[sea_orm(has_many)]
    pub documents: HasMany<super::documents::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
