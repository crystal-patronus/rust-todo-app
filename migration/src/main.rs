use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    let key = "";
    if std::env::var(key).is_err() {
        // Getting the database URL from Rocket.toml if it's not set
        let figment = rocket::Config::figment();
        let database_url: String = figment
            .extract_inner("databases.todo.url")
            .expect("Cannot find Database URL in Rocket.toml");
        std::env::set_var(key, database_url);
    }
    
    cli::run_cli(migration::Migrator).await;
}
