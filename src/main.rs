use axum::{Router, routing::{get, put, delete, patch}, response::Html, http::StatusCode};
use clap::Parser;

mod kv;
mod http_kv;
mod server_state;
use kv::KeyValue;
use http_kv::{http_get, http_set, http_delete, http_set_empty_key, http_unsupported, get_server_info};
use kv::now_secs;
use server_state::ServerState;
use server_state::clean_expired;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Name of the host machine
    #[arg(short = 'H', long, default_value_t = "localhost".to_string())]
    host: String,

    /// Name of the rack
    #[arg(short = 'r', long, default_value_t = "default".to_string())]
    rack: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let state = ServerState::new(args.host, args.rack);
    tokio::spawn(clean_expired(state.clone()));

    let app = Router::new()
        .route("/", get(root))
        .route("/kv/:key", get(http_get))
        .route("/kv/:key", put(http_set).post(http_set))
        .route("/kv/:key", delete(http_delete))
        .route("/kv/:key", patch(http_unsupported))
        .route("/kv/", get(http_set_empty_key).put(http_set_empty_key).delete(http_set_empty_key).post(http_unsupported).patch(http_unsupported))
        // .route("/clear", delete(http_clear))
        // .route("/clear", get(http_unsupported).put(http_unsupported).post(http_unsupported).patch(http_unsupported))
        // .route("/table", delete(http_clear))
        // .route("/table", get(http_get_table_version))
        // .route("/table", put(http_unsupported).post(http_unsupported).patch(http_unsupported))
        .route("/health", get(healthcheck))
        .route("/health", put(http_unsupported).post(http_unsupported).patch(http_unsupported).delete(http_unsupported))
        .route("/info", get(get_server_info))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn healthcheck() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Healthy")
}
