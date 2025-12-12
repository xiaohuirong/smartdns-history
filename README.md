# SmartDNS History

A real-time web dashboard for visualizing [SmartDNS](https://github.com/pymumu/smartdns) audit logs. This tool parses the `smartdns-audit.log` file, provides a live feed of DNS queries via WebSockets, and allows for filtering and inspection of historical records.

## Features

- **Real-time Updates**: DNS queries appear instantly as they occur.
- **Historical View**: Loads existing log entries on startup and buffers recent history.
- **Filtering**:
  - **Domain**: Search for specific visited domains.
  - **Client IP**: Filter requests by source IP.
  - **Type**: Filter by DNS record type (e.g., `1` for A, `28` for AAAA).
  - **Group**: Filter by SmartDNS server group (e.g., `china`, `default`).
- **Pause/Resume**: Pause the live feed to inspect specific logs without losing background updates.
- **Responsive UI**: Clean, single-page interface built with Tailwind CSS.

## Tech Stack

- **Backend**: Rust
  - [Axum](https://github.com/tokio-rs/axum): Web application framework.
  - [Tokio](https://tokio.rs/): Asynchronous runtime.
  - [Linemux](https://github.com/jbe/linemux): File tailing library.
  - [Clap](https://github.com/clap-rs/clap): Command-line argument parsing.
- **Frontend**: HTML5, JavaScript (ES6), Tailwind CSS (CDN).

## Installation & Build

### Prerequisites
- [Rust Toolchain](https://www.rust-lang.org/tools/install) (cargo, rustc)

### Build
```bash
git clone https://github.com/yourusername/smartdns-history.git
cd smartdns-history
cargo build --release
```

The binary will be located at `target/release/smartdns-history`.

## Usage

Run the application pointing to your SmartDNS audit log file.

### Basic Usage
By default, it looks for `smartdns-audit.log` in the current directory and listens on port `3000`.

```bash
./target/release/smartdns-history
```

### Custom Configuration

You can specify the log file path and the server port using command-line arguments.

```bash
# Specifying a custom log file path
./target/release/smartdns-history --log /var/log/smartdns/smartdns-audit.log

# Specifying a custom port
./target/release/smartdns-history --port 8080

# Both
./target/release/smartdns-history --log /var/log/smartdns/smartdns-audit.log --port 8080
```

### Accessing the Dashboard
Open your web browser and navigate to:
`http://localhost:3000` (or your configured port).

## Project Structure

```
.
├── assets/
│   └── index.html      # Single-page frontend application
├── src/
│   ├── main.rs         # Application entry, server setup, and WebSocket logic
│   ├── model.rs        # Data structures (LogEntry)
│   └── parser.rs       # Log line parsing logic (Regex)
├── Cargo.toml          # Rust dependencies
└── README.md           # This file
```

## License

MIT
