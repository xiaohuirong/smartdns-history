# GEMINI Project Context

This document serves as a context guide for LLMs (like Gemini) or developers working on the `smartdns-history` codebase. It outlines the architectural decisions, data flow, and key components.

## Architecture Overview

The application is a single-binary Rust web server that functions as a log visualizer. It follows a Producer-Consumer pattern using Tokio's broadcast channels.

### Core Components

1.  **Log Watcher (Producer)**:
    - Located in `src/main.rs` (`watch_log` function).
    - **Initialization**: Reads the existing log file line-by-line to populate the initial history buffer.
    - **Tailing**: Uses the `linemux` crate to watch for *new* lines appended to the log file asynchronously.
    - **Parsing**: Delegates to `src/parser.rs` to convert raw log strings into structured `LogEntry` objects.
    - **Broadcasting**: Sends parsed entries to a `tokio::sync::broadcast` channel.

2.  **State Management**:
    - `AppState` struct holds:
        - `tx`: The broadcast sender channel.
        - `history`: A `RwLock<VecDeque<LogEntry>>` that acts as a ring buffer (size 200) to store recent logs for new client connections.

3.  **Web Server (Axum)**:
    - Serves static content (the embedded `index.html`).
    - Handles WebSocket upgrades at `/ws`.

4.  **WebSocket Handler (Consumer)**:
    - On connection, sends the current `history` (JSON array) to the client immediately.
    - Subscribes to the broadcast channel.
    - Streams new `LogEntry` objects to the client as JSON as they arrive.

5.  **Frontend**:
    - Pure HTML/JS located in `assets/index.html`.
    - Included in the binary via `include_str!` macro for single-file distribution.
    - Uses Tailwind CSS via CDN for styling.
    - Maintains its own client-side buffer (size 1000) and handles filtering logic in the browser.

## Data Models

**`src/model.rs`**:
```rust
pub struct LogEntry {
    pub timestamp: String,
    pub client_ip: String,
    pub domain: String,
    pub qtype: u16,
    pub time_ms: u64,
    pub speed_ms: f64,
    pub group: String,
    pub result: String,
}
```

## Key Files & Logic

-   **`src/main.rs`**:
    -   `main()`: Sets up the Tokio runtime, loads configuration (CLI > Config File > Defaults), initializes shared state, spawns the log watcher task, and starts the Axum server.
    -   `ws_handler()`: Upgrades HTTP connection to WebSocket.
    -   `handle_socket()`: Manages the WebSocket session (sending history + streaming updates).

-   **`config.toml`**:
    -   Optional configuration file for setting the log file path and server port.
    -   Loaded via the `config` crate.

-   **`src/parser.rs`**:
    -   Contains a compiled `Regex` (using `OnceLock`) to efficiently parse log lines.
    -   Format expected: `[TIMESTAMP] IP query DOMAIN, type QTYPE, time TIMEms, speed: SPEEDms, group GROUP, result RESULT`

## Development

### Running Locally
```bash
# Run with default settings (creates/watches smartdns-audit.log in CWD)
cargo run

# Run with logging enabled
RUST_LOG=info cargo run
```

### Building
```bash
cargo build --release
```

## Potential Future Improvements

-   **Embedded Assets**: Currently `index.html` is included via `include_str!`. For more complex assets, `rust-embed` could be used.
-   **Server-side Filtering**: Move filtering logic to the backend if log volume becomes too high for the client to handle.
-   **Persistent Storage**: Use a lightweight DB (SQLite) instead of in-memory `VecDeque` if long-term history is needed.
