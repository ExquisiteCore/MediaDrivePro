use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250305_000003_create_files_table::Files;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MediaInfo::Table)
                    .if_not_exists()
                    .col(pk_uuid(MediaInfo::Id))
                    .col(uuid(MediaInfo::FileId).not_null().unique_key())
                    .col(string_len(MediaInfo::MediaType, 16).not_null())
                    .col(string_len_null(MediaInfo::Title, 255))
                    .col(integer_null(MediaInfo::Season))
                    .col(integer_null(MediaInfo::Episode))
                    .col(integer_null(MediaInfo::TmdbId))
                    .col(string_len_null(MediaInfo::PosterUrl, 512))
                    .col(text_null(MediaInfo::Overview))
                    .col(integer_null(MediaInfo::Year))
                    .col(integer_null(MediaInfo::Duration))
                    .col(string_len_null(MediaInfo::Resolution, 16))
                    .col(
                        timestamp_with_time_zone(MediaInfo::CreatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(MediaInfo::Table, MediaInfo::FileId)
                            .to(Files::Table, Files::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_media_info_tmdb")
                    .table(MediaInfo::Table)
                    .col(MediaInfo::TmdbId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MediaInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MediaInfo {
    Table,
    Id,
    FileId,
    MediaType,
    Title,
    Season,
    Episode,
    TmdbId,
    PosterUrl,
    Overview,
    Year,
    Duration,
    Resolution,
    CreatedAt,
}
