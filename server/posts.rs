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
    pub tag: String,
}

#[derive(Deserialize)]
pub struct VoteForm {
    pub post_id: i64,
    pub vote: i64,
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

    let tag = match form.tag.as_str() {
        "General" | "Web" | "Crypto" | "Rev" | "Pwn" | "Forensics" | "OSINT" | "Misc" => form.tag.as_str(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                "Neteisingas tagas debile lerva",
            ).into_response();
        }
    };

    let result = sqlx::query("INSERT INTO posts (username, content, tag) VALUES (?, ?, ?)")
        .bind(&username)
        .bind(trimmed)
        .bind(tag)
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

pub async fn vote_post(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
    Form(form): Form<VoteForm>,
) -> axum::response::Response {

    let username = match jar.get("username") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Redirect::to("/login").into_response();
        }
    };

    if form.vote != 1 && form.vote != -1 {
        return (
            StatusCode::BAD_REQUEST,
            "Invalidas.",
        ).into_response();
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR
                .into_response();
        }
    };

    let deleted = sqlx::query(
        "DELETE FROM votes
        WHERE post_id = ?
        AND username = ?
        AND vote = ?"
    )
    .bind(form.post_id)
    .bind(&username)
    .bind(form.vote)
    .execute(&mut *tx)
    .await;

    match deleted {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let result = sqlx::query(
                    "INSERT INTO votes (post_id, username, vote)
                    VALUES (?, ?, ?)
                    ON CONFLICT(post_id, username)
                    DO UPDATE SET vote = excluded.vote"
                )
                .bind(form.post_id)
                .bind(&username)
                .bind(form.vote)
                .execute(&mut *tx)
                .await;

                if result.is_err() {
                    return (
                        StatusCode::BAD_REQUEST,
                        "Failed to vote :)",
                    ).into_response();
                }
            }
        }
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR
                .into_response();
        }
    }

    if tx.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR
            .into_response();
    }
    Redirect::to("/recs").into_response()
}

pub async fn render_posts(pool: &SqlitePool, username: Option<&str>) -> String {
    let posts: Vec<(String, String, String, i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT posts.username, posts.content, COALESCE(posts.tag, 'Misc'), COALESCE(users.avatar, 1),
        posts.id,
        COALESCE((SELECT SUM(vote) FROM votes
            WHERE votes.post_id = posts.id), 0),
        COALESCE((SELECT vote FROM votes
            WHERE votes.post_id = posts.id
            AND votes.username = ?), 0)
        FROM posts 
        LEFT JOIN users ON posts.username = users.username 
        ORDER BY posts.id DESC"
    )
    .bind(username)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut posts_html = String::new();
    for (author, content, tag, avatar, id, score, user_vote) in posts {
        let avatar_src = format!("../res/avatars/pic{}.png", avatar);
        let safe_tag = escape_html(&tag);
        let upvote_class = if user_vote == 1 { "active" } else { "" };
        let downvote_class = if user_vote == -1 { "active" } else { "" };
        
        let score_class = match user_vote {
            1 => "positive",
            -1 => "negative",
            _ => "",
        };

        posts_html.push_str(&format!(
            r#"<section class="post-box">
                <div class="post-user">
                    <img src="{}" alt="avatar" class="post-avatar">
                    <span>@{}</span>
                    <span class="post-tag {}">{}</span>
                </div>
                <p class="post-content">{}</p>

                <form action="/vote" method="POST" class="post-votes">
                    <input type="hidden" name="post_id" value="{id}">
                    <button type="submit" name="vote" value="1"
                        class="vote-btn upvote {upvote_class}"
                        aria-label="Upvote">
                        <i class="bxf bx-arrow-big-up"></i>
                    </button>
                    
                    <span class="vote-score {score_class}">{score}</span>

                    <button type="submit" name="vote" value="-1"
                        class="vote-btn downvote {downvote_class}"
                        aria-label="Downvote">
                        <i class="bxf bx-arrow-big-down"></i>
                    </button>
                    </form>
            </section>"#,
            avatar_src,
            escape_html(&author),
            safe_tag,
            safe_tag,
            escape_html(&content)
        ));
    }

    posts_html
}

pub async fn render_user_posts(pool: &SqlitePool, username: &str) -> String {
    let posts: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT posts.username, posts.content, COALESCE(posts.tag, 'tag1'), COALESCE(users.avatar, 1)
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
    for (author, content, tag, avatar) in posts {
        let avatar_src = format!("../res/avatars/pic{}.png", avatar);
        let safe_tag = escape_html(&tag);

        posts_html.push_str(&format!(
            r#"<section class="post-box">
                <div class="post-user">
                    <img src="{}" alt="avatar" class="post-avatar">
                    <span>@{}</span>
                    <span class="post-tag {}">{}</span>
                </div>
                <p class="post-content">{}</p>
            </section>"#,
            avatar_src,
            escape_html(&author),
            safe_tag,
            safe_tag,
            escape_html(&content)
        ));
    }

    if posts_html.is_empty() {
        posts_html = "<p>You haven't posted anything yet stupid moid go contribute to the statistic</p>".to_string();
    }
    
    posts_html
}