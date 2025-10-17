use axum::{
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tracing::{error, info, Level};

mod netlink_ports;

use netlink_ports::PortInfo;

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiRequest {
    data: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Build the application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/api/hello", get(hello))
        .route("/api/data", post(post_data))
        .route("/api/ports", get(get_ports))
        .route("/sse", get(sse_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    info!(
        "devcon-remote listening on {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "devcon-remote REST API with SSE and netlink support"
}

async fn hello() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Hello from devcon-remote!".to_string(),
    })
}

async fn post_data(Json(payload): Json<ApiRequest>) -> Json<ApiResponse> {
    Json(ApiResponse {
        message: format!("Received: {}", payload.data),
    })
}

async fn get_ports() -> Json<Vec<PortInfo>> {
    match netlink_ports::get_listening_ports().await {
        Ok(ports) => Json(ports),
        Err(e) => {
            error!("Failed to get ports: {}", e);
            Json(vec![])
        }
    }
}

async fn sse_handler(
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
        .map(|_| Ok(Event::default().data("ping")));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
