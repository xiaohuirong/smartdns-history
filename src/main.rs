mod model;
mod parser;

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use clap::Parser;
use config::{Config, File};
use futures::StreamExt;
use linemux::MuxedLines;
use serde::Deserialize;
use std::{collections::VecDeque, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tracing::info;

// Config
const HISTORY_SIZE: usize = 200; // Keep last 200 lines in backend memory for new clients

#[derive(Debug, Deserialize)]
struct Settings {
    log_file: PathBuf,
    port: u16,
}

impl Settings {
    fn new(config_path: Option<PathBuf>, args: &Args) -> Result<Self, config::ConfigError> {
        let mut builder = Config::builder();

        // 1. Default values
        builder = builder
            .set_default("log_file", "smartdns-audit.log")?
            .set_default("port", 3000)?;

        // 2. Config file
        if let Some(path) = config_path {
             builder = builder.add_source(File::from(path));
        } else {
             // Try standard locations
             let etc_path = PathBuf::from("/etc/smartdns-history/config.toml");
             if etc_path.exists() {
                 builder = builder.add_source(File::from(etc_path));
             } else {
                 // Try local config.toml
                 let local_path = PathBuf::from("config.toml");
                 if local_path.exists() {
                     builder = builder.add_source(File::from(local_path));
                 }
             }
        }

        // 3. Environment variables
        builder = builder.add_source(config::Environment::with_prefix("SMARTDNS_HISTORY"));

        let mut settings: Settings = builder.build()?.try_deserialize()?;

        // 4. CLI Overrides
        if let Some(log) = &args.log {
            settings.log_file = log.clone();
        }
        if let Some(port) = args.port {
            settings.port = port;
        }

        Ok(settings)
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Path to the smartdns audit log file
    #[arg(short, long)]
    log: Option<PathBuf>,

    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,
}

struct AppState {
    tx: broadcast::Sender<model::LogEntry>,
    history: RwLock<VecDeque<model::LogEntry>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    
    let settings = match Settings::new(args.config.clone(), &args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    // Broadcast channel
    let (tx, _) = broadcast::channel(1000);

    // State
    let state = Arc::new(AppState {
        tx: tx.clone(),
        history: RwLock::new(VecDeque::with_capacity(HISTORY_SIZE)),
    });

    println!("Starting up...");
    info!("Using log file: {:?}", settings.log_file);

    // Start log watcher
    let state_clone = state.clone();
    let log_path = settings.log_file.clone();
    tokio::spawn(async move {
        watch_log(state_clone, log_path).await;
    });

    // Router
    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], settings.port));
    info!("SmartDNS History listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../assets/index.html"))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    // Send history first
    let history = state.history.read().await;
    // We send the history as a single JSON array message to distinguish from real-time updates (which are single objects)
    // Or we can just iterate and send one by one. Sending one by one is simpler for the client logic if it just appends.
    for entry in history.iter() {
        if let Ok(json) = serde_json::to_string(entry) {
             if socket.send(Message::Text(json)).await.is_err() {
                 return;
             }
        }
    }
    drop(history); // release lock

    // Stream updates
    while let Ok(msg) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&msg) {
            if socket.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    }
}

async fn watch_log(state: Arc<AppState>, log_path: PathBuf) {
    // Ensure the log file exists.
    if !log_path.exists() {
        info!("Log file {:?} not found. Creating it...", log_path);
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::File::create(&log_path).expect("Failed to create/touch log file");
    }

    // Read existing content into history
    info!("Reading existing log entries from {:?}", log_path);
    if let Ok(file) = tokio::fs::File::open(&log_path).await {
        use tokio::io::{self, AsyncBufReadExt};
        let reader = io::BufReader::new(file);
        let mut lines_stream = reader.lines();

        while let Some(line_result) = lines_stream.next_line().await.transpose() {
            match line_result {
                Ok(line) => {
                    if let Some(entry) = parser::parse_line(&line) {
                        let mut history = state.history.write().await;
                        if history.len() >= HISTORY_SIZE {
                            history.pop_front();
                        }
                        history.push_back(entry);
                        // No need to broadcast old entries, they are part of initial history.
                    }
                },
                Err(e) => {
                    info!("Error reading historical line: {}", e);
                    break;
                }
            }
        }
    } else {
        info!("Could not open log file {:?} for initial read.", log_path);
    }
    info!("Finished reading existing log entries.");


    // Set up linemux to watch for new lines
    let mut lines = MuxedLines::new().expect("Failed to create muxed lines");
    lines.add_file(&log_path).await.expect("Failed to add file to linemux");
    info!("Watching {:?} for new entries.", log_path);

    while let Some(Ok(line)) = lines.next().await {
        if let Some(entry) = parser::parse_line(line.line()) {
            // Update history
            let mut history = state.history.write().await;
            if history.len() >= HISTORY_SIZE {
                history.pop_front();
            }
            history.push_back(entry.clone());
            
            // Broadcast
            let _ = state.tx.send(entry);
        }
    }
}