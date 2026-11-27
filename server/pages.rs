//visa page loadinimo logika
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use sqlx::SqlitePool;
use std::fs;

//cookieszz.. mnjhjhgggggggguhhh

async fn render_page(
    file_path: &str,
    jar: &CookieJar,
    pool: &SqlitePool,
) -> Html<String> {
    let html_content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| "<h1>bleh</h1>".to_string());

    //apsauga nuo injetionu
    let user_html = if let Some(cookie) = jar.get("username") {
        let safe_name = cookie
            .value()
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"',"&quot;")
            .replace('\'', "&#39;");

        let avatar: Option<i64> = sqlx::query_scalar(
            "SELECT avatar FROM users WHERE username =?"
        )
        .bind(cookie.value())
        .fetch_one(pool)
        .await
        .unwrap_or(None);

        if let Some(avatar) = avatar {
            fs::read_to_string("static/profile.html")
            .unwrap_or_default()
            .replace("{{AVATAR}}", &avatar.to_string())
            .replace("{{USERNAME}}", &safe_name)
        } else {
            String::new()
        }

    } else {
        String::new() // guest :p
    };

    let rendered = html_content.replace("{{USER_SECTION}}", &user_html);
    Html(rendered)
}

pub async fn index_handler(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
) -> Html<String> {
    render_page("static/index.html", &jar, &pool).await
}

pub async fn about_page(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
) -> Html<String> {
    render_page("static/about.html", &jar, &pool).await
}

pub async fn changelog_page(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
) -> Html<String> {
    render_page("static/changelog.html", &jar, &pool).await
}

pub async fn recs_page(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
) -> Html<String> {
    let username = jar.get("username").map(|cookie| cookie.value());
    let posts_html = crate::posts::render_posts(&pool, username).await;
    let mut page = render_page("static/recs.html", &jar, &pool).await.0;
    page = page.replace("{{POSTS_SECTION}}", &posts_html);
    Html(page)
}

pub async fn writeups_page(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
) -> Html<String> {
    render_page("static/writeups.html", &jar, &pool).await
}

pub async fn history_page(
    jar: CookieJar,
    State(pool): State<SqlitePool>,
) -> Html<String> {
    let posts_html = if let Some(cookie) = jar.get("username") {
        crate::posts::render_user_posts(&pool, cookie.value()).await
    } else {
        "<p>You must log in to view your history faggy ahh noob not even faggy honestly just weird tbh it's genuinely weird dude wtf dude ur so weird oh my genuine god ur so weird fricking foid</p>".to_string()
    };
    let mut page = render_page("static/history.html", &jar, &pool).await.0;
    page = page.replace("{{POSTS_SECTION}}", &posts_html);
    Html(page)
}

pub async fn login_page() -> Html<String>{
    let html_content = fs::read_to_string("static/login.html")
        .unwrap_or_else(|_| "<h1>bleh</h1>".to_string());
    Html(html_content)
}

pub async fn register_page() -> Html<String>{
    let html_content: String = fs::read_to_string("static/register.html")
        .unwrap_or_else(|_| "<h1>bleh</h1>".to_string());
    Html(html_content)
}
