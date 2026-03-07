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
                    .table(Files::Table)
                    .if_not_exists()
                    .col(pk_uuid(Files::Id))
                    .col(uuid(Files::UserId).not_null())
                    .col(uuid_null(Files::FolderId))
                    .col(string_len(Files::Name, 255).not_null())
                    .col(string_len(Files::StorageKey, 512).not_null())
                    .col(big_integer(Files::Size).not_null())
                    .col(string_len(Files::ContentType, 128).not_null())
                    .col(string_len(Files::HashSha256, 64).not_null())
                    .col(
                        string_len(Files::StorageBackend, 32)
                            .not_null()
                            .default("fs"),
                    )
                    .col(string_len(Files::Status, 16).not_null().default("active"))
                    .col(
                        timestamp_with_time_zone(Files::CreatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .col(
                        timestamp_with_time_zone(Files::UpdatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Files::Table, Files::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Files::Table, Files::FolderId)
                            .to(Folders::Table, Folders::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_files_user_folder")
                    .table(Files::Table)
                    .col(Files::UserId)
                    .col(Files::FolderId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_files_hash")
                    .table(Files::Table)
                    .col(Files::HashSha256)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Files {
    Table,
    Id,
    UserId,
    FolderId,
    Name,
    StorageKey,
    Size,
    ContentType,
    HashSha256,
    StorageBackend,
    Status,
    CreatedAt,
    UpdatedAt,
}
