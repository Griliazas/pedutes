use axum::{
    extract::{Form,State},
    http::StatusCode,
    response::{Html, Redirect, IntoResponse},
    routing::get,
    Router,
};

use std::fs;
use serde::Deserialize;
use tower_http::services::ServeDir;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
    },
    Argon2,
};
async fn index_handler() -> Html<String>{
    let html_content = fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| "<h1>bleh</h1>".to_string());
    Html(html_content)
}

async fn login_page() -> Html<String>{
    let html_content = fs::read_to_string("static/login.html")
        .unwrap_or_else(|_| "<h1>bleh</h1>".to_string());
    Html(html_content)
}

async fn register_page() -> Html<String>{
    let html_content: String = fs::read_to_string("static/register.html")
        .unwrap_or_else(|_| "<h1>bleh</h1>".to_string());
    Html(html_content)
}

//sitas tsg kad galetu html formas i string paverst

#[derive(Deserialize)]
struct UserForm {
    username: String,
    password: String,
}

//registracija

async fn register_user(
    State(pool): State<SqlitePool>,
    Form(form): Form<UserForm>,
) -> axum::response::Response {

    //tikrinam data kuria submittina

    if form.username.trim().is_empty()
        || form.username.len() > 32
        || form.password.len() < 8
        || form.password.len() > 128
    {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid username or password :3",
        ).into_response();
    }

    //kaip pas pona pascali ir ai destytoja, sugeneruojam salt ir td hashinam slaptikus

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match Argon2::default()
        .hash_password(form.password.as_bytes(), &salt)
    {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not process registration :("
            ).into_response();
        }
    };
    
    //ikisam musu data i sql ughh~~~

    let result = sqlx::query(
        "INSERT INTO users (username, password_hash)
        VALUES (?, ?)"
    )
    .bind(&form.username)
    .bind(&password_hash)
    .execute(&pool)
    .await;

    match result {
    Ok(_) => Redirect::to("/").into_response(),

    Err(sqlx::Error::Database(err))
        if err.is_unique_violation() =>
    {
        (
            StatusCode::CONFLICT,
            "Username already exists :(",
        ).into_response()
    }

    Err(_) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Database error :(",
    ).into_response(),
}
}

//cia ipisau logina ahujena

async fn login_user(
    State(pool):State<SqlitePool>,
    Form(form): Form<UserForm>,
) -> axum::response::Response {
    
    //pagal slapyvardi rasim vierchus

    let result: Result<Option<(String,)>, _> =
        sqlx::query_as(
            "SELECT password_hash FROM users
            WHERE username = ?"
        )
        .bind(&form.username)
        .fetch_optional(&pool)
        .await;

        //checkinam ar bico slapyvardis vps egzituoja

    let stored_hash = match result {
        Ok(Some((hash,))) => hash,

        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                "Invalid username or password :(",
            ).into_response();
        }

        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error :(",
        ).into_response();
    }
};

    //pakeiciam hasha i argon2

    let parsed_hash = match PasswordHash::new(&stored_hash) {
    Ok(hash) => hash,

    Err(_) => {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Password verification error :(",
        ).into_response();
    }
};
    
    //tikrinam ar slaptiks teisings

    let password_correct = Argon2::default()
        .verify_password(
            form.password.as_bytes(),
            &parsed_hash,
        )
        .is_ok();

    if password_correct {
        Redirect::to("/").into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            "Invalid username or password >:(",
        ).into_response()
}
}

#[tokio::main]
async fn main() {

    //cia jungiam duombaze :)

    let options = SqliteConnectOptions::new()
        .filename("forum.db")
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options)
        .await
        .expect("Failed to connect to database :3");

    //cia musu sql table susikuria
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        )"
    )

    .execute(&pool)
    .await
    .expect("Failed to create table :3");

    println!("Database connected :3");

    // nenukopijavau sito is google trust
    let app = Router::new()

        .route("/", get(index_handler))
        .route("/login", get(login_page).post(login_user))
        .route("/register",get(register_page).post(register_user))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/res", ServeDir::new("res")) //jei nori pridet nauja folder turi linkint sitame
        .with_state(pool);

    // as zinau kaip sita veikia yes ahahaha
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("http://127.0.0.1:3000");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
