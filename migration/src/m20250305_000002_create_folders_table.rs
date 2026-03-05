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
                    .table(Folders::Table)
                    .if_not_exists()
                    .col(pk_uuid(Folders::Id))
                    .col(uuid(Folders::UserId).not_null())
                    .col(uuid_null(Folders::ParentId))
                    .col(string_len(Folders::Name, 255).not_null())
                    .col(
                        timestamp_with_time_zone(Folders::CreatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()".to_string()),
                    )
                    .col(
                        timestamp_with_time_zone(Folders::UpdatedAt)
                            .not_null()
                            .extra("DEFAULT NOW()".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Folders::Table, Folders::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Folders::Table, Folders::ParentId)
                            .to(Folders::Table, Folders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique constraint: no duplicate names in the same parent folder for the same user
        manager
            .create_index(
                Index::create()
                    .name("idx_folders_unique_name")
                    .table(Folders::Table)
                    .col(Folders::UserId)
                    .col(Folders::ParentId)
                    .col(Folders::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_folders_parent")
                    .table(Folders::Table)
                    .col(Folders::UserId)
                    .col(Folders::ParentId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Folders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Folders {
    Table,
    Id,
    UserId,
    ParentId,
    Name,
    CreatedAt,
    UpdatedAt,
}
