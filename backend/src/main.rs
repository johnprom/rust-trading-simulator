use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let static_files = ServeDir::new("../frontend/target/dx/frontend/release/web/public")
        .not_found_service(axum::routing::get(|| async {
            axum::response::Html(include_str!("../../frontend/target/dx/frontend/release/web/public/index.html"))
        }));

    let app = Router::new()
        .fallback_service(static_files);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
