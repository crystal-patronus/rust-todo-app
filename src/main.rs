#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::response::{Responder, Result as ResponseResult};
use rocket::Request;
use rocket_db_pools::{Connection, Database};
use rocket::http::Status;
use sqlx::{self};

#[derive(Deserialize, Serialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde")]
struct Task {
    id: i64,
    item: String
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskItem<'r> {
    item: &'r str
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskId {
    id: u8
}

#[derive(Database)]
#[database("todo")]
struct TodoDatabase(rocket_db_pools::sqlx::PgPool);

struct DatabaseError(rocket_db_pools::sqlx::Error);

impl<'r> Responder<'r, 'r> for DatabaseError {
    fn respond_to(self, _: &Request<'_>) -> ResponseResult<'r> {
        Err(Status::InternalServerError)
    }
}

impl From<rocket_db_pools::sqlx::Error> for DatabaseError {
    fn from(error: rocket_db_pools::sqlx::Error) -> Self {
        DatabaseError(error)
    }
}

#[get("/")]
fn index() -> &'static str {
    "hello, world!"
}

#[allow(unused)]
#[post("/addtask", data="<task>")]
async fn add_task(task: Json<TaskItem<'_>>, mut db: Connection<TodoDatabase>) -> Result<Json<Task>, DatabaseError> {
    let added_task = sqlx::query_as::<_, Task>("INSERT INTO tasks (item) VALUES ($1) RETURNING *")
        .bind(task.item)
        .fetch_one(&mut **db)
        .await?;

    Ok(Json(added_task))
}

#[get("/readtasks")]
async fn read_tasks(mut db: Connection<TodoDatabase>) -> Result<Json<Vec<Task>>, DatabaseError> {
    let all_tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks")
        .fetch_all(&mut **db)
        .await?;

    Ok(Json(all_tasks))
}

#[get("/edittask", data="<task_update>")]
async fn edit_task(task_update: Json<Task>, mut db: Connection<TodoDatabase>) -> Result<Json<Task>, DatabaseError> {
    let updated_task = sqlx::query_as::<_, Task>("UPDATE tasks SET item = $1 WHERE id = $2 RETURNING *")
        .bind(&task_update.item)
        .bind(task_update.id)
        .fetch_one(&mut **db)
        .await?;

    Ok(Json(updated_task))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(TodoDatabase::init())
        .mount("/", routes![index, add_task, read_tasks, edit_task])
}
