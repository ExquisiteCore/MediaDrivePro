use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "media_info")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub file_id: Uuid,
    pub media_type: String,
    pub title: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub tmdb_id: Option<i32>,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
    pub year: Option<i32>,
    pub duration: Option<i32>,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::files::Entity",
        from = "Column::FileId",
        to = "super::files::Column::Id"
    )]
    File,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
