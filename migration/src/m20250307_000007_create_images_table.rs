use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250305_000001_create_users_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Images::Table)
                    .if_not_exists()
                    .col(pk_uuid(Images::Id))
                    .col(uuid(Images::UserId).not_null())
                    .col(string_len(Images::HashSha256, 64).not_null().unique_key())
                    .col(string_len(Images::OriginalName, 255).not_null())
                    .col(string_len(Images::StorageKey, 512).not_null())
                    .col(string_len(Images::ThumbKey, 512).not_null())
                    .col(big_integer(Images::Size).not_null())
                    .col(big_integer(Images::OriginalSize).not_null())
                    .col(integer(Images::Width).not_null())
                    .col(integer(Images::Height).not_null())
                    .col(string_len(Images::ContentType, 128).not_null().default("image/webp"))
                    .col(
                        timestamp_with_time_zone(Images::CreatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Images::Table, Images::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_images_user")
                    .table(Images::Table)
                    .col(Images::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Images::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Images {
    Table,
    Id,
    UserId,
    HashSha256,
    OriginalName,
    StorageKey,
    ThumbKey,
    Size,
    OriginalSize,
    Width,
    Height,
    ContentType,
    CreatedAt,
}
