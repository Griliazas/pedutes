use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // nenukopijavau sito is google trust
    let app = Router::new().route("/", get(|| async { "uh aah uhhhhh labas" }));

    // as zinau kaip sita veikia yes ahahaha
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("http://127.0.0.1:3000");
    
    axum::serve(listener, app).await.unwrap();
}
