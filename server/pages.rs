//visa page loadinimo logika
use axum::response::Html;
use axum_extra::extract::cookie::CookieJar;
use std::fs;

//cookieszz.. mnjhjhgggggggguhhh

fn render_page(file_path: &str, jar: &CookieJar) -> Html<String> {
    let html_content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| "<h1>bleh</h1>".to_string());

    //apsauga nuo injetionu
    let user_html = if let Some(cookie) = jar.get("username") {
        let safe_name = cookie
            .value()
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        format!("<span>{}</span>", safe_name)
    } else {
        String::new() // guest :p
    };

    let rendered = html_content.replace("{{USER_SECTION}}", &user_html);
    Html(rendered)
}

pub async fn index_handler(jar: CookieJar) -> Html<String> {
    render_page("static/index.html", &jar)
}

pub async fn about_page(jar: CookieJar) -> Html<String> {
    render_page("static/about.html", &jar)
}

pub async fn changelog_page(jar: CookieJar) -> Html<String> {
    render_page("static/changelog.html", &jar)
}

pub async fn recs_page(jar: CookieJar) -> Html<String> {
    render_page("static/recs.html", &jar)
}

pub async fn writeups_page(jar: CookieJar) -> Html<String> {
    render_page("static/writeups.html", &jar)
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
