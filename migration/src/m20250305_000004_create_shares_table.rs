use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250305_000001_create_users_table::Users;
use crate::m20250305_000002_create_folders_table::Folders;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Shares::Table)
                    .if_not_exists()
                    .col(pk_uuid(Shares::Id))
                    .col(uuid(Shares::UserId).not_null())
                    .col(uuid_null(Shares::FileId))
                    .col(uuid_null(Shares::FolderId))
                    .col(string_len(Shares::Token, 32).unique_key().not_null())
                    .col(string_len_null(Shares::Password, 64))
                    .col(
                        string_len(Shares::Permission, 16)
                            .not_null()
                            .default("read"),
                    )
                    .col(integer_null(Shares::MaxDownloads))
                    .col(integer(Shares::DownloadCount).not_null().default(0))
                    .col(timestamp_with_time_zone_null(Shares::ExpiresAt))
                    .col(
                        timestamp_with_time_zone(Shares::CreatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Shares::Table, Shares::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Shares::Table, Shares::FileId)
                            .to(Files::Table, Files::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Shares::Table, Shares::FolderId)
                            .to(Folders::Table, Folders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shares_token")
                    .table(Shares::Table)
                    .col(Shares::Token)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Shares::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Shares {
    Table,
    Id,
    UserId,
    FileId,
    FolderId,
    Token,
    Password,
    Permission,
    MaxDownloads,
    DownloadCount,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
pub(crate) enum Files {
    Table,
    Id,
}
