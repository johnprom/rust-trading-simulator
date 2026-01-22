mod api_client;
mod db;
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

    // Initialize database
    let db_path = "/app/data/trading_sim.db";
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| format!("sqlite:{}", db_path));

    tracing::info!("Connecting to database: {}", database_url);
    tracing::info!("Database file path: {}", db_path);

    // Ensure data directory exists and check permissions
    match std::fs::create_dir_all("/app/data") {
        Ok(_) => tracing::info!("Data directory exists/created successfully"),
        Err(e) => tracing::error!("Failed to create data directory: {}", e),
    }

    // Check directory permissions
    match std::fs::metadata("/app/data") {
        Ok(metadata) => {
            tracing::info!("Directory /app/data exists, permissions: {:?}", metadata.permissions());
        }
        Err(e) => {
            tracing::error!("Cannot access /app/data: {}", e);
        }
    }

    // Try to create a test file
    match std::fs::write("/app/data/test.txt", "test") {
        Ok(_) => {
            tracing::info!("Successfully wrote test file to /app/data");
            let _ = std::fs::remove_file("/app/data/test.txt");
        }
        Err(e) => {
            tracing::error!("Cannot write to /app/data: {}", e);
        }
    }

    let db = db::Database::new(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    tracing::info!("Running database migrations...");
    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database initialized successfully");

    // Initialize application state
    let state = AppState::new(db).await;

    // Spawn price polling task
    let polling_state = state.clone();
    tokio::spawn(async move {
        services::price_service::start_price_polling(polling_state).await;
    });

    let api_routes = Router::new()
        .route("/price", get(routes::price::get_price))
        .route("/price/history", get(routes::price::get_price_history))
        .route("/portfolio", get(routes::portfolio::get_portfolio))
        .route("/trade", post(routes::trade::post_trade))
        .route("/signup", post(routes::auth::signup))
        .route("/login", post(routes::auth::login));

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
