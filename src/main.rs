#[macro_use] extern crate rocket;

use std::{fs::OpenOptions, io::{ BufReader, BufRead, Write }};
use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Task<'r> {
    item: &'r str
}

#[get("/")]
fn index() -> &'static str {
    "hello, world!"
}

#[get("/readtasks")]
fn read_tasks() -> Json<Vec<String>> {
    let tasks = OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open("tasks.txt")
                    .expect("unable to access tasks.txt");
    let reader = BufReader::new(tasks);
    Json(reader.lines()
            .map(|line| line.expect("could no read line"))
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
    let task_item_string = format!("{}\n", task.item);
    let task_item_bytes = task_item_string.as_bytes();
    tasks.write(task_item_bytes).expect("unable to write to tasks.txt");
    "Task added successfully"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, add_task, read_tasks])
}
