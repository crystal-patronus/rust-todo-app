#[macro_use] extern crate rocket;
extern crate argon2;

mod pool;

use migration::{tests_cfg::json, MigratorTrait};
use entity::{tasks, users::{self, USER_PASSWORD_SALT}};
use entity::tasks::Entity as Tasks;
use entity::users::Entity as Users;
use pool::Db;

use argon2::Config;
use rocket::{
    request::{Outcome, FromRequest, FlashMessage},
    response::{Responder, Flash, Redirect, Result as ResponseResult},
    fairing::{AdHoc, self},
    fs::{FileServer, relative},
    serde::json::Json,
    http::{CookieJar, Cookie, Status},
    form::Form,
    Request,
    Rocket,
    Build
};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, ColumnTrait, QueryOrder, QueryFilter, Set}; // DeleteResult
use sea_orm_rocket::{Connection, Database};
use rocket_dyn_templates::Template;

#[allow(dead_code)]
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


#[get("/?<page>&<tasks_per_page>")]
async fn index(
    conn: Connection<'_, Db>,
    flash: Option<FlashMessage<'_>>,
    page: Option<usize>,
    tasks_per_page: Option<usize>,
    user: AuthenticatedUser
) -> Result<Template, DatabaseError> {
    let db = conn.into_inner();
    let page = page.unwrap_or(0);
    let tasks_per_page = tasks_per_page.unwrap_or(5);

    let paginator = Tasks::find()
        .filter(tasks::Column::UserId.eq(user.user_id))
        .order_by_asc(tasks::Column::Id)
        .paginate(db, tasks_per_page);
    let number_of_pages = paginator.num_pages().await?;
    let tasks = paginator.fetch_page(page).await?;

    Ok(Template::render(
        "todo_list",
        json!({
            "tasks": tasks,
            "flash": flash.map(FlashMessage::into_inner),
            "number_of_pages": number_of_pages,
            "current_page": page,
            "tasks_per_page": tasks_per_page,
        })
    ))
}

#[allow(unused)]
#[get("/?<page>&<tasks_per_page>", rank = 2)]
async fn index_redirect(page: Option<usize>, tasks_per_page: Option<usize>) -> Redirect {
    redirect_to_login()
}

fn redirect_to_login() -> Redirect {
    Redirect::to("/login")
}

#[post("/addtask", data="<task_form>")]
async fn add_task(conn: Connection<'_, Db>, task_form: Form<tasks::Model>, user: AuthenticatedUser) -> Flash<Redirect> {
    let db = conn.into_inner();
    let task = task_form.into_inner();

    let active_task: tasks::ActiveModel = tasks::ActiveModel {
        item: Set(task.item),
        user_id: Set(user.user_id),
        ..Default::default()
    };

    match active_task.insert(db).await {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue creating the task");
        }
    };

    Flash::success(Redirect::to("/"), "Task created!")
}

#[post("/addtask", rank = 2)]
async fn add_task_redirect() -> Redirect {
    redirect_to_login()
}

#[get("/readtasks")]
async fn read_tasks(conn: Connection<'_, Db>) -> Result<Json<Vec<tasks::Model>>, DatabaseError> {
    let db = conn.into_inner();

    Ok(Json(
        Tasks::find()
            .order_by_asc(tasks::Column::Id)
            .all(db)
            .await?
    ))
}

#[put("/edittask", data="<task_form>")]
async fn edit_task(conn: Connection<'_, Db>, task_form: Form<tasks::Model>, _user: AuthenticatedUser) -> Flash<Redirect> {
    let db = conn.into_inner();
    let task = task_form.into_inner();

    let task_to_update = match Tasks::find_by_id(task.id).one(db).await {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue editing the task");
        }
    };
    let mut task_to_update:tasks::ActiveModel = task_to_update.unwrap().into();
    task_to_update.item = Set(task.item);
    match task_to_update.update(db).await {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue editing the task");
        }
    };

    Flash::success(Redirect::to("/"), "Task edited successfully!")
}

#[put("/edittask", rank = 2)]
async fn edit_task_redirect() -> Redirect {
    redirect_to_login()
}

#[get("/edit/<id>")]
async fn edit_task_page(conn: Connection<'_, Db>, id: i32, _user: AuthenticatedUser) -> Result<Template, DatabaseError> {
    let db = conn.into_inner();
    let task = Tasks::find_by_id(id).one(db).await?.unwrap();

    Ok(Template::render(
        "edit_task_form",
        json!({
            "task": task
        })
    ))
}

#[allow(unused)]
#[get("/edit/<id>", rank = 2)]
async fn edit_task_page_redirect(id: i32) -> Redirect {
    redirect_to_login()
}

#[delete("/deletetask/<id>")]
async fn delete_task(conn: Connection<'_, Db>, id: i32, _user: AuthenticatedUser) -> Flash<Redirect> {
    let db = conn.into_inner();
    let _result = match Tasks::delete_by_id(id).exec(db).await {
        Ok(value) => value,
        Err(_) => {
            return Flash::error(Redirect::to("/"), "Issue deleting the task");
        }
    };

    // Ok(format!("{} task(s) deleted", result.rows_affected))
    Flash::success(Redirect::to("/"), "Task successfully deleted!")
}

#[allow(unused)]
#[delete("/deletetask/<id>", rank = 2)]
async fn delete_task_redirect(id: i32) -> Redirect {
    redirect_to_login()
}

#[get("/login")]
async fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "login_page",
        json!({
            "flash": flash.map(FlashMessage::into_inner)
        })
    )
}

#[post("/logout")]
async fn logout(cookies: & CookieJar<'_>) -> Flash<Redirect> {
    remove_user_id_cookie(cookies);
    Flash::success(Redirect::to("/login"), "Logged out succesfully!")
}

#[get("/signup")]
async fn signup_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "signup_page",
        json!({
            "flash": flash.map(FlashMessage::into_inner)
        })
    )
}

#[allow(dead_code)]
struct AuthenticatedUser {
    user_id: i32
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = anyhow::Error;
    
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let user_id_cookie = match get_user_id_cookie(cookies) {
            Some(result) => result,
            None => return Outcome::Forward(Status::BadRequest)
        };

        let logged_in_user_id = match user_id_cookie.value()
            .parse::<i32>() {
                Ok(result) => result,
                Err(_err) => return Outcome::Forward(Status::BadRequest)
            };

        return Outcome::Success(AuthenticatedUser {
            user_id: logged_in_user_id
        })
    }
}

#[post("/createaccount", data="<user_form>")]
async fn create_account(conn: Connection<'_, Db>, user_form: Form<users::Model>) -> Flash<Redirect> {
    let db = conn.into_inner();
    let user = user_form.into_inner();

    let hash_config = Config::default();
    let hash = match argon2::hash_encoded(user.password.as_bytes(), USER_PASSWORD_SALT, &hash_config) {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/signup"), "Issue creating account")
        }
    };

    let active_user = users::ActiveModel {
        username: Set(user.username),
        password: Set(hash),
        ..Default::default()
    };

    match active_user.insert(db).await {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/signup"), "Issue creating account");
        }
    };

    Flash::success(Redirect::to("/login"), "Account created successfully!")
}

fn get_user_id_cookie<'a>(cookies: &'a CookieJar) -> Option<Cookie<'a>> {
    cookies.get_private("user_id")
}

fn set_user_id_cookie(cookies: & CookieJar, user_id: i32) {
    cookies.add_private(Cookie::new("user_id", user_id.to_string()));
}

fn remove_user_id_cookie(cookies: & CookieJar) {
    cookies.remove_private(Cookie::from("user_id"))
}

fn login_error() -> Flash<Redirect> {
    Flash::error(Redirect::to("/login"), "Incorrect username or password")
}

#[post("/verifyaccount", data="<user_form>")]
async fn verify_account(conn: Connection<'_, Db>, cookies: & CookieJar<'_>, user_form: Form<users::Model>) -> Flash<Redirect> {
    let db = conn.into_inner();
    let user = user_form.into_inner();

    let stored_user = match Users::find()
        .filter(users::Column::Username.contains(&user.username))
        .one(db)
        .await {
            Ok(model_or_null) => {
                match model_or_null {
                    Some(model) => model,
                    None => {
                        return login_error();
                    }
                }
            },
            Err(_) => {
                return login_error();
            }
        };
    
    let is_password_correct = match argon2::verify_encoded(&stored_user.password, user.password.as_bytes()) {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/login"), "Encountered an issue processing your account")
        }
    };

    if !is_password_correct {
        return login_error();
    }

    set_user_id_cookie(cookies, stored_user.id);
    Flash::success(Redirect::to("/"), "Logged in succesfully!")
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
        .mount("/", FileServer::from(relative!("/public")))
        .mount("/", routes![
            index,
            index_redirect,
            add_task,
            add_task_redirect,
            read_tasks,
            edit_task,
            edit_task_redirect,
            delete_task,
            delete_task_redirect,
            edit_task_page,
            edit_task_page_redirect,
            login_page,
            logout,
            signup_page,
            create_account,
            verify_account]
        )
        .attach(Template::fairing())
}
