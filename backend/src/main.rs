mod api_client;
mod models;
mod routes;
mod services;
mod state;

use axum::{routing::{get, post}, Router};
use state::AppState;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new();

    // Spawn price polling task
    let polling_state = state.clone();
    tokio::spawn(async move {
        services::price_service::start_price_polling(polling_state).await;
    });

    let api_routes = Router::new()
        .route("/price", get(routes::price::get_price))
        .route("/portfolio", get(routes::portfolio::get_portfolio))
        .route("/trade", post(routes::trade::post_trade));

    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// use axum::{
//     Router,
//     routing::get,
// };
// use tower_http::services::ServeDir;
// use std::net::SocketAddr;

// #[tokio::main]
// async fn main() {
//     // Serve files from "static" directory
//     let static_files = ServeDir::new("static");

//     // Build the router
//     let app = Router::new().nest_service("/", static_files);

//     // Bind and serve
//     let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
//     println!("Listening on {}", addr);

//     let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
//     axum::serve(listener, app).await.unwrap();
// }
