```rust
use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;
use async_trait::async_trait;
use std::sync::Mutex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use chrono::{DateTime, Utc, TimeZone};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FitnessProgress {
    id: u64,
    user_id: u64,
    progress: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    progress_records: HashMap<u64, FitnessProgress>,
    users: HashMap<u64, User>,
}

impl Database {
    fn new() -> Self {
        Self {
            progress_records: HashMap::new(),
            users: HashMap::new(),
        }
    }

    // CRUD DATA
    fn insert_progress(&mut self, progress: FitnessProgress) {
        self.progress_records.insert(progress.id, progress);
    }

    fn get_progress(&self, id: &u64) -> Option<&FitnessProgress> {
        self.progress_records.get(id)
    }

    fn get_all_progress(&self) -> Vec<&FitnessProgress> {
        self.progress_records.values().collect()
    }

    fn delete_progress(&mut self, id: &u64) {
        self.progress_records.remove(id);
    }

    fn update_progress(&mut self, progress: FitnessProgress) {
        self.progress_records.insert(progress.id, progress);
    }

    // USER DATA RELATED FUNCTIONS
    fn insert_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users.values().find(|u| u.username == username)
    }

    // DATABASE SAVING
    fn save_to_file(&self) -> std::io::Result<()> {
        let data: String = serde_json::to_string(&self)?;
        let mut file: fs::File = fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file_content: String = fs::read_to_string("database.json")?;
        let db: Database = serde_json::from_str(&file_content)?;
        Ok(db)
    }
}

struct AppState {
    db: Mutex<Database>,
}

async fn create_progress(app_state: web::Data<AppState>, progress: web::Json<FitnessProgress>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.insert_progress(progress.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn read_progress(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    match db.get_progress(&id.into_inner()) {
        Some(progress) => HttpResponse::Ok().json(progress),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn read_all_progress(app_state: web::Data<AppState>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    let progress_records = db.get_all_progress();
    HttpResponse::Ok().json(progress_records)
}

async fn update_progress(app_state: web::Data<AppState>, progress: web::Json<FitnessProgress>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.update_progress(progress.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn delete_progress(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.delete_progress(&id.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn register(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.insert_user(user.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn login(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    match db.get_user_by_name(&user.username) {
        Some(stored_user) if stored_user.password == user.password => {
            HttpResponse::Ok().body("Logged in!")
        }
        _ => HttpResponse::BadRequest().body("Invalid username or password"),
    }
}

async fn fetch_time() -> Result<DateTime<Utc>, reqwest::Error> {
    let client = HttpClient::new();
    let res = client.get("http://worldtimeapi.org/api/timezone/Etc/UTC")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let datetime_str = res["utc_datetime"].as_str().unwrap();
    let datetime = DateTime::parse_from_rfc3339(datetime_str).unwrap().with_timezone(&Utc);
    Ok(datetime)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db: Database = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new(),
    };

    let data: web::Data<AppState> = web::Data::new(AppState {
        db: Mutex::new(db),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(data.clone())
            .route("/progress", web::post().to(create_progress))
            .route("/progress", web::get().to(read_all_progress))
            .route("/progress", web::put().to(update_progress))
            .route("/progress/{id}", web::get().to(read_progress))
            .route("/progress/{id}", web::delete().to(delete_progress))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/time", web::get().to(fetch_time))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```