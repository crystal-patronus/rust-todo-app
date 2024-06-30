#[macro_use] extern crate rocket;

mod pool;

use migration::MigratorTrait;
use entity::tasks;

use pool::Db;
use rocket::{
    response::{Responder, Result as ResponseResult},
    fairing::{AdHoc, self},
    serde::json::Json,
    http::Status,
    form::Form,
    Request,
    Rocket,
    Build
};
use sea_orm::{ActiveModelTrait, Set}; //, EntityTrait, QueryOrder, DeleteResult};
use sea_orm_rocket::{Connection, Database};

struct DatabaseError(sea_orm::DbErr);

impl<'r> Responder<'r, 'r> for DatabaseError {
    fn respond_to(self, _: &Request<'_>) -> ResponseResult<'r> {
        Err(Status::InternalServerError)
    }
}

impl From<sea_orm::DbErr> for DatabaseError {
    fn from(error: sea_orm::DbErr) -> Self {
        DatabaseError(error)
    }
}


#[get("/")]
fn index() -> &'static str {
    "hello, world!"
}

#[post("/addtask", data="<task_form>")]
async fn add_task(conn: Connection<'_, Db>, task_form: Form<tasks::Model>) -> Result<Json<tasks::Model>, DatabaseError> {
    let db = conn.into_inner();
    let task = task_form.into_inner();

    let active_task: tasks::ActiveModel = tasks::ActiveModel {
        item: Set(task.item),
        ..Default::default()
    };

    Ok(Json(active_task.insert(db).await?))
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migration::Migrator::up(conn, None).await;
    Ok(rocket)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/", routes![index, add_task])
}
