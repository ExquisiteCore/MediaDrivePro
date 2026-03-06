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
                    .table(ApiTokens::Table)
                    .if_not_exists()
                    .col(pk_uuid(ApiTokens::Id))
                    .col(uuid(ApiTokens::UserId).not_null())
                    .col(string_len(ApiTokens::Name, 64).not_null())
                    .col(string_len(ApiTokens::TokenHash, 64).unique_key().not_null())
                    .col(
                        string_len(ApiTokens::Permissions, 255)
                            .not_null()
                            .default("read,write"),
                    )
                    .col(timestamp_with_time_zone_null(ApiTokens::ExpiresAt))
                    .col(timestamp_with_time_zone_null(ApiTokens::LastUsedAt))
                    .col(
                        timestamp_with_time_zone(ApiTokens::CreatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ApiTokens::Table, ApiTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_api_tokens_user")
                    .table(ApiTokens::Table)
                    .col(ApiTokens::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum ApiTokens {
    Table,
    Id,
    UserId,
    Name,
    TokenHash,
    Permissions,
    ExpiresAt,
    LastUsedAt,
    CreatedAt,
}
