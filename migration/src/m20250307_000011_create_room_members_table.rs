use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250305_000001_create_users_table::Users;
use crate::m20250307_000010_create_rooms_table::Rooms;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RoomMembers::Table)
                    .if_not_exists()
                    .col(uuid(RoomMembers::RoomId).not_null())
                    .col(uuid(RoomMembers::UserId).not_null())
                    .col(
                        string_len(RoomMembers::Role, 16)
                            .not_null()
                            .default("member"),
                    )
                    .col(
                        timestamp_with_time_zone(RoomMembers::JoinedAt)
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string()),
                    )
                    .primary_key(
                        Index::create()
                            .col(RoomMembers::RoomId)
                            .col(RoomMembers::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RoomMembers::Table, RoomMembers::RoomId)
                            .to(Rooms::Table, Rooms::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(RoomMembers::Table, RoomMembers::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RoomMembers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RoomMembers {
    Table,
    RoomId,
    UserId,
    Role,
    JoinedAt,
}
