use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240709_062721_create_users_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Username).string().not_null())
                    .col(ColumnDef::new(Users::Password).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(
                sea_query::Table::drop()
                .table(Users::Table)
                .to_owned()
            )
            .await
    }
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Username,
    Password,
}
