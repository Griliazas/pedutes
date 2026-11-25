use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct PostForm {
    pub content: String,
}
//as parasysiu komentartus kita komita
fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub async fn create_post(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
    Form(form): Form<PostForm>,
) -> axum::response::Response {
    let username = match jar.get("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "prisijunk loxas",
            ).into_response();
        }
    };

    //kad nerasytu debilisku postu limita galesim pakeist
    let trimmed = form.content.trim();
    if trimmed.len() > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            "ne benai ne jis per didelis!",
        ).into_response();
    }
    else if trimmed.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "nu kazka bent parasyk",
        ).into_response();
    }

    let result = sqlx::query("INSERT INTO posts (username, content) VALUES (?, ?)")
        .bind(&username)
        .bind(trimmed)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Redirect::to("/recs").into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save post.",
        ).into_response(),
    }
}

pub async fn render_posts(pool: &SqlitePool) -> String {
    let posts: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT posts.username, posts.content, COALESCE(users.avatar, 1) 
         FROM posts 
         LEFT JOIN users ON posts.username = users.username 
         ORDER BY posts.id DESC"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut posts_html = String::new();
    for (author, content, avatar) in posts {
        let avatar_src = format!("../res/avatars/pic{}.png", avatar);

        posts_html.push_str(&format!(
            r#"<section class="post-box">
                <div class="post-user">
                    <img src="{}" alt="avatar" class="post-avatar">
                    <span>@{}</span>
                </div>
                <p class="post-content">{}</p>
            </section>"#,
            avatar_src,
            escape_html(&author),
            escape_html(&content)
        ));
    }

    posts_html
}

pub async fn render_user_posts(pool: &SqlitePool,username: &str,) -> String {
    let posts: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT posts.username, posts.content, COALESCE(users.avatar, 1)
         FROM posts
         LEFT JOIN users ON posts.username = users.username
         WHERE posts.username = ?
         ORDER BY posts.id DESC"
    )
    .bind(username)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut posts_html = String::new();
    for (author, content, avatar) in posts {
        let avatar_src = format!("../res/avatars/pic{}.png", avatar);

        posts_html.push_str(&format!(
            r#"<section class="post-box">
                <div class="post-user">
                    <img src="{}" alt="avatar" class="post-avatar">
                    <span>@{}</span>
                </div>
                <p class="post-content">{}</p>
            </section>"#,
            avatar_src,
            escape_html(&author),
            escape_html(&content)
        ));
    }

    if posts_html.is_empty() {
        posts_html = "<p>You haven't posted anything yet RETARD IEVA YOU FOID FUCKING FISH :p</p>"
            .to_string();
    }
    
    posts_html
}