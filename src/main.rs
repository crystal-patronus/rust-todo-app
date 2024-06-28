#[macro_use] extern crate rocket;

use std::{fs::OpenOptions, io::{ BufReader, BufRead, Write }};
use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Task<'r> {
    item: &'r str
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskUpdate<'r> {
    id: u8,
    item: &'r str
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TaskId {
    id: i64
}

#[get("/")]
fn index() -> &'static str {
    "hello, world!"
}

#[get("/readtasks")]
fn read_task() -> Json<Vec<String>> {
    let tasks = OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open("tasks.txt")
                    .expect("unable to access tasks.txt");
    let reader = BufReader::new(tasks);
    Json(reader.lines()
            .map(|line| {
                let line_string: String = line.expect("could not read line");
                let line_pieces: Vec<&str> = line_string.split(",").collect();
                line_pieces[1].to_string()
            })
            .collect())
}

#[post("/addtask", data="<task>")]
fn add_task(task: Json<Task<'_>>) -> &'static str {
    let mut tasks = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open("tasks.txt")
                        .expect("unable to access tasks.txt");
    let reader = BufReader::new(&tasks);
    let id = reader.lines().count();
    let task_item_string = format!("{},{}\n", id, task.item);
    let task_item_bytes = task_item_string.as_bytes();
    tasks.write(task_item_bytes).expect("unable to write to tasks.txt");
    "Task added successfully"
}

#[put("/edittask", data="<task_update>")]
fn edit_task(task_update: Json<TaskUpdate<'_>>) -> &'static str {
    let tasks = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open("tasks.txt")
                        .expect("unable to access tasks.txt");
    let mut temp = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open("temp.txt")
                        .expect("unable to access temp.txt");
    
    let reader = BufReader::new(tasks);
    for line in reader.lines() {
        let line_string: String = line.expect("could not read line");
        let line_pieces: Vec<&str> = line_string.split(",").collect();
        if line_pieces[0].parse::<u8>().expect("unable to parse id as u8") == task_update.id {
            let task_items: [&str; 2] = [line_pieces[0], task_update.item];
            let task = format!("{}\n", task_items.join(","));
            temp.write(task.as_bytes()).expect("could not write to temp file");
        }
        else {
            let task = format!("{}\n", line_string);
            temp.write(task.as_bytes()).expect("could not write to temp file");
        }
    }

    std::fs::remove_file("tasks.txt").expect("unable to remove tasks.txt");
    std::fs::rename("temp.txt", "tasks.txt").expect("unable to renmae temp.txt");
    "Task updated successfully"
}

#[delete("/deletetask", data="<task_id>")]
fn delte_task(task_id: Json<TaskId>) -> &'static str {
    let tasks = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open("tasks.txt")
                        .expect("unable to access tasks.txt");
    let mut temp = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open("temp.txt")
                        .expect("unable to access temp.txt");
                    let reader = BufReader::new(tasks);

    for line in reader.lines() {
        let line_string: String = line.expect("could not read line");
        let line_pieces: Vec<&str> = line_string.split(",").collect();
        if line_pieces[0].parse::<u8>().expect("unable to parse id as u8") != task_id.id {
            let task = format!("{}\n", line_string);
            temp.write(task.as_bytes()).expect("could not write to temp file");
        }
    }

    std::fs::remove_file("tasks.txt").expect("unable to remove tasks.txt");
    std::fs::rename("temp.txt", "tasks.txt").expect("unable to renmae temp.txt");
    "Task deleted successfully"
}

#[get("/readtasks")]
async fn read_tasks(mut db: Connection<TodoDatabase>) -> Result<Json<Vec<Task>>, DatabaseError> {
    let all_tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks")
        .fetch_all(&mut **db)
        .await?;

    Ok(Json(all_tasks))
}

#[put("/edittask", data="<task_update>")]
async fn edit_task(task_update: Json<Task>, mut db: Connection<TodoDatabase>) -> Result<Json<Task>, DatabaseError> {
    let updated_task = sqlx::query_as::<_, Task>("UPDATE tasks SET item = $1 WHERE id = $2 RETURNING *")
        .bind(&task_update.item)
        .bind(task_update.id)
        .fetch_one(&mut **db)
        .await?;

    Ok(Json(updated_task))
}

#[delete("/deletetask", data="<task_id>")]
async fn delete_task(task_id: Json<TaskId>, mut db: Connection<TodoDatabase>) -> Result<Json<Task>, DatabaseError> {
    let deleted_task = sqlx::query_as::<_, Task>("DELETE FROM tasks WHERE id = $1 RETURNING *")
        .bind(task_id.id)
        .fetch_one(&mut **db)
        .await?;

    Ok(Json(deleted_task))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(TodoDatabase::init())
        .mount("/", routes![index, add_task, read_tasks, edit_task, delete_task])

}
