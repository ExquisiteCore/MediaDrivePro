use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub storage_key: String,
    pub size: i64,
    pub content_type: String,
    pub hash_sha256: String,
    pub storage_backend: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::folders::Entity",
        from = "Column::FolderId",
        to = "super::folders::Column::Id"
    )]
    Folder,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::folders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Folder.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
