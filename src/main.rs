use axum::{
    response::Html,
    routing::get,
    Router,
};

use std::fs;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {

    //html thing
    async fn index_handler() -> Html<String> {
        let html_content = fs::read_to_string("static/index.html")
            .unwrap_or_else(|_| "<h1>bleh</h1>".to_string());
        Html(html_content)
    }

    // nenukopijavau sito is google trust
    let app = Router::new()

        .route("/", get(index_handler))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/res", ServeDir::new("res")); //jei nori pridet nauja folder turi linkint sitame

    // as zinau kaip sita veikia yes ahahaha
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("http://127.0.0.1:3000");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
