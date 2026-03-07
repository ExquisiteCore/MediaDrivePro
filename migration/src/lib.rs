pub use sea_orm_migration::prelude::*;

mod m20250305_000001_create_users_table;
mod m20250305_000002_create_folders_table;
mod m20250305_000003_create_files_table;
mod m20250305_000004_create_shares_table;
mod m20250306_000005_create_api_tokens_table;
mod m20250307_000006_add_avatar_to_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250305_000001_create_users_table::Migration),
            Box::new(m20250305_000002_create_folders_table::Migration),
            Box::new(m20250305_000003_create_files_table::Migration),
            Box::new(m20250305_000004_create_shares_table::Migration),
            Box::new(m20250306_000005_create_api_tokens_table::Migration),
            Box::new(m20250307_000006_add_avatar_to_users::Migration),
        ]
    }

    fn migration_table_name() -> sea_orm::DynIden {
        Alias::new("_migrations").into_iden()
    }
}
