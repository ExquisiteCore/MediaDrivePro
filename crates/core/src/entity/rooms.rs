use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rooms")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub host_id: Uuid,
    pub name: String,
    pub invite_code: String,
    pub status: String,
    pub current_file_id: Option<Uuid>,
    pub current_time: f64,
    pub max_members: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::HostId",
        to = "super::users::Column::Id"
    )]
    Host,
    #[sea_orm(
        belongs_to = "super::files::Entity",
        from = "Column::CurrentFileId",
        to = "super::files::Column::Id"
    )]
    CurrentFile,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Host.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
