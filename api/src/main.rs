//! REST API for Windows Failover Cluster and Hyper-V management
//!
//! Supports running as:
//! - Console application (default)
//! - Windows Service (with --service flag)
//!
//! Configuration is loaded from config.toml in the executable directory.

use std::sync::Arc;

use api::{create_router, init_tracing, service::windows_service, AppState, Config};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check if running as Windows service
    if args.iter().any(|arg| arg == "--service") {
        run_as_service();
    } else {
        run_console();
    }
}

fn run_as_service() {
    // Load config to get the service name
    let config_path = windows_service::get_config_path();
    let config = Config::load(&config_path).unwrap_or_default();

    #[cfg(windows)]
    {
        if let Err(e) = windows_service::run_as_service(&config.service.name) {
            eprintln!("Service error: {}", e);
            std::process::exit(1);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = config; // silence unused warning
        eprintln!("Windows service mode is only available on Windows");
        std::process::exit(1);
    }
}

fn run_console() {
    // Load configuration
    let config = Config::load("config.toml").unwrap_or_else(|e| {
        eprintln!("Warning: {}", e);
        Config::default()
    });

    // Initialize tracing
    init_tracing(&config.logging.level);

    tracing::info!(
        "Starting API server on {}:{}",
        config.server.host,
        config.server.port
    );

    // Create tokio runtime and run
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    runtime.block_on(async {
        let state = Arc::new(AppState);
        let app = create_router(state);

        let addr: std::net::SocketAddr = config
            .socket_addr()
            .parse()
            .expect("Invalid socket address");

        tracing::info!("API server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");

        axum::serve(listener, app).await.expect("Server error");
    });
}
