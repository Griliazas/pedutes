#[path = "../server/user.rs"]
mod user;

#[path = "../server/pages.rs"]
mod pages;

#[path = "../server/posts.rs"]
mod posts;

//rs failu locations ^^^ ir deklaravimas

use axum::{
    routing::{get, post},
    Router,
};

use tower_http::services::ServeDir;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

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

    //tikrinam ar avatar column egzistuoja

    let avatar_column_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('users')
         WHERE name = 'avatar'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check avatar column");

    //jeigu neegzistuoja, pridedam

    if avatar_column_exists == 0 {
        sqlx::query(
            "ALTER TABLE users
             ADD COLUMN avatar INTEGER NOT NULL DEFAULT 1
             CHECK (avatar IN (1, 2, 3))"
        )
        .execute(&pool)
        .await
        .expect("Failed to add avatar column");
    }

    //posts table

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            content TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .expect("Failed to create posts table :3");

    println!("Database connected :3");

    // prideta del daug psl username ir uhh dabar nav kitaip rasomi jo
    //sitame gali paziet kaip callinamos funkcijos is kitu failu :p
let app = Router::new()
    .route("/", get(pages::index_handler))
    .route("/post", post(posts::create_post))
    .route("/about", get(pages::about_page))
    .route("/changelog", get(pages::changelog_page))
    .route("/recs", get(pages::recs_page))
    .route("/writeups", get(pages::writeups_page))
    .route("/login", get(pages::login_page).post(user::login_user))
    .route("/register", get(pages::register_page).post(user::register_user))
    .route("/logout", get(user::logout_user))
    .route("/avatar", post(user::change_avatar))
    .nest_service("/static", ServeDir::new("static"))
    .nest_service("/res", ServeDir::new("res"))
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