//authenticator logika :p +cookie
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use sqlx::SqlitePool;

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

//logout
pub async fn logout_user(jar: CookieJar) -> impl IntoResponse {
    let mut cookie = Cookie::from("username");
    cookie.set_path("/");
    (jar.remove(cookie), Redirect::to("/"))
}

//sitas tsg kad galetu html formas i string paverst

#[derive(Deserialize)]
pub struct UserForm {
    username: String,
    password: String,
}

//registracija

pub async fn register_user(
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
    Ok(_) => (
        StatusCode::CREATED,
        "Registration successful!"
    ).into_response(),

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

pub async fn login_user(
    jar: CookieJar,
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
        let mut cookie = Cookie::new("username", form.username);
        cookie.set_path("/");
        cookie.set_http_only(true);

        // sorey kad pakeiciau tavo redirect del cookie nepyk
        (jar.add(cookie), Redirect::to("/")).into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            "Invalid username or password >:(",
        ).into_response()
}
}

#[derive(Deserialize)]
pub struct AvatarForm{
    avatar: i64,
}

pub async fn change_avatar(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
    Form(form): Form<AvatarForm>,
) -> axum::response::Response {
    let username = match jar.get("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "You must be logged in GRR"
            ).into_response();
        }
    };

    if !(1..=3).contains(&form.avatar) {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid profile picture"
        ).into_response();
    }

    let result = sqlx::query(
        "UPDATE users SET avatar = ?
        WHERE username =?"
    )
    .bind(form.avatar)
    .bind(&username)
    .execute(&pool)
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => (
            StatusCode::OK,
            "Profile picture updated :3"
        ).into_response(),

        Ok(_) => (
            StatusCode::UNAUTHORIZED,
            "User not found :("
        ).into_response(),

        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Yayy database error"
        ).into_response(),
    }
}