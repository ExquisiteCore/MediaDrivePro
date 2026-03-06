use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_uuid(Users::Id))
                    .col(string_len(Users::Username, 64).unique_key().not_null())
                    .col(string_len(Users::Email, 255).unique_key().not_null())
                    .col(string_len(Users::Password, 255).not_null())
                    .col(string_len(Users::Role, 16).not_null().default("user"))
                    .col(big_integer(Users::StorageQuota).not_null().default(10737418240i64))
                    .col(big_integer(Users::StorageUsed).not_null().default(0))
                    .col(
                        timestamp_with_time_zone(Users::CreatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .col(
                        timestamp_with_time_zone(Users::UpdatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Users {
    Table,
    Id,
    Username,
    Email,
    Password,
    Role,
    StorageQuota,
    StorageUsed,
    CreatedAt,
    UpdatedAt,
}
