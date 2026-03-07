use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250305_000001_create_users_table::Users;
use crate::m20250305_000003_create_files_table::Files;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Rooms::Table)
                    .if_not_exists()
                    .col(pk_uuid(Rooms::Id))
                    .col(uuid(Rooms::HostId).not_null())
                    .col(string_len(Rooms::Name, 128).not_null())
                    .col(string_len(Rooms::InviteCode, 16).unique_key().not_null())
                    .col(
                        string_len(Rooms::Status, 16)
                            .not_null()
                            .default("waiting"),
                    )
                    .col(uuid_null(Rooms::CurrentFileId))
                    .col(float(Rooms::CurrentTime).not_null().default(0.0))
                    .col(integer(Rooms::MaxMembers).not_null().default(20))
                    .col(
                        timestamp_with_time_zone(Rooms::CreatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .col(
                        timestamp_with_time_zone(Rooms::UpdatedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Rooms::Table, Rooms::HostId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Rooms::Table, Rooms::CurrentFileId)
                            .to(Files::Table, Files::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Rooms::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Rooms {
    Table,
    Id,
    HostId,
    Name,
    InviteCode,
    Status,
    CurrentFileId,
    CurrentTime,
    MaxMembers,
    CreatedAt,
    UpdatedAt,
}
