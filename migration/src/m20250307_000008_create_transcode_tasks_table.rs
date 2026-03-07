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
                    .table(TranscodeTasks::Table)
                    .if_not_exists()
                    .col(pk_uuid(TranscodeTasks::Id))
                    .col(uuid(TranscodeTasks::FileId).not_null())
                    .col(
                        string_len(TranscodeTasks::Status, 16)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        string_len(TranscodeTasks::Profile, 32)
                            .not_null()
                            .default("720p"),
                    )
                    .col(small_integer(TranscodeTasks::Progress).not_null().default(0))
                    .col(string_len_null(TranscodeTasks::OutputKey, 512))
                    .col(text_null(TranscodeTasks::ErrorMsg))
                    .col(small_integer(TranscodeTasks::RetryCount).not_null().default(0))
                    .col(timestamp_with_time_zone_null(TranscodeTasks::StartedAt))
                    .col(timestamp_with_time_zone_null(TranscodeTasks::CompletedAt))
                    .col(
                        timestamp_with_time_zone(TranscodeTasks::CreatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TranscodeTasks::Table, TranscodeTasks::FileId)
                            .to(Files::Table, Files::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_transcode_status")
                    .table(TranscodeTasks::Table)
                    .col(TranscodeTasks::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_transcode_file")
                    .table(TranscodeTasks::Table)
                    .col(TranscodeTasks::FileId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TranscodeTasks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TranscodeTasks {
    Table,
    Id,
    FileId,
    Status,
    Profile,
    Progress,
    OutputKey,
    ErrorMsg,
    RetryCount,
    StartedAt,
    CompletedAt,
    CreatedAt,
}
