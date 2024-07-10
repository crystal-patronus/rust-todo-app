pub use sea_orm_migration::prelude::*;

mod m20240630_063944_create_tasks_table;
mod m20240709_062721_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240709_062721_create_users_table::Migration),
            Box::new(m20240630_063944_create_tasks_table::Migration),
        ]
    }
}
