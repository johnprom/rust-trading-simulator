use axum::{
    Router,
    routing::get,
};
use tower_http::services::ServeDir;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Serve files from "static" directory
    let static_files = ServeDir::new("static");

    // Build the router
    let app = Router::new().nest_service("/", static_files);

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
