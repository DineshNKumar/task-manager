use actix_web::{App, HttpResponse, HttpServer, Responder, delete, get, patch, post, web};

use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use tasky::models::task::{GetTask, Task, TaskPayload};

#[get("/")]
async fn get_tasks(db: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as!(
        GetTask,
        r#"
            SELECT * FROM tasks
            "#
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(tasks) => HttpResponse::Ok().json(tasks),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch tasks")
        }
    }
}

#[post("/")]
async fn create_task(db: web::Data<PgPool>, payload: web::Json<Task>) -> impl Responder {
    let result = sqlx::query!(
        r#"
            INSERT INTO tasks (title, description, due_date, is_completed)
            VALUES ($1, $2, $3, $4)
            "#,
        payload.title,
        payload.description,
        payload.due_date,
        payload.is_completed
    )
    .execute(db.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Created().json("Task created successfully"),
        Err(e) => {
            eprintln!("Error inserting task: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to create task")
        }
    }
}

#[patch("/{id}")]
async fn update_task(
    db: web::Data<PgPool>,
    path: web::Path<i32>,
    payload: web::Json<TaskPayload>,
) -> impl Responder {
    let task_id = path.into_inner();
    let task = payload.into_inner();

    let result = sqlx::query!(
        r#"
            UPDATE tasks
            SET
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                is_completed = COALESCE($3, is_completed),
                due_date = COALESCE($4, due_date),
                updated_at = now()
            WHERE id = $5
            "#,
        task.title,
        task.description,
        task.is_completed,
        task.due_date,
        task_id
    )
    .execute(db.get_ref())
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => HttpResponse::Ok().body("Task updated successfully."),
        Ok(_) => HttpResponse::NotFound().body("Task not found."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to update: {}", e)),
    }
}

#[delete("/{id}")]
async fn delete_task(db: web::Data<PgPool>, path: web::Path<i32>) -> impl Responder {
    let task_id = path.into_inner();

    let result = sqlx::query!(
        r#"
            DELETE FROM tasks WHERE id = $1
            "#,
        task_id
    )
    .execute(db.get_ref())
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => HttpResponse::Ok().body("Task deleted successfully."),
        Ok(_) => HttpResponse::NotFound().body("Task not found."),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to update: {}", e)),
    }
}

#[get("/csv")]
async fn get_tasks_in_csv(db: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as!(
        GetTask,
        r#"
               SELECT * FROM tasks
           "#
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(tasks) => {
            let mut wtr = csv::Writer::from_writer(vec![]);

            for task in tasks {
                if let Err(e) = wtr.serialize(task) {
                    eprintln!("CSV serialization error: {:?}", e);
                    return HttpResponse::InternalServerError()
                        .body("Failed to serialize task to CSV");
                }
            }

            match wtr.into_inner() {
                Ok(data) => HttpResponse::Ok()
                    .append_header(("Content-Type", "text/csv"))
                    .append_header(("Content-Disposition", "attachment; filename=\"tasks.csv\""))
                    .body(data),
                Err(e) => {
                    eprintln!("CSV writer error: {:?}", e);
                    HttpResponse::InternalServerError().body("Failed to write CSV")
                }
            }
        }
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch tasks")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let port = env::var("PORT")
        .expect("PORT not set")
        .parse::<u16>()
        .unwrap();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    println!("✅ Connected to Postgres!");

    println!("Tasky is running on http://127.0.0.1:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(create_task)
            .service(get_tasks)
            .service(update_task)
            .service(delete_task)
            .service(get_tasks_in_csv)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await?;

    Ok(())
}
