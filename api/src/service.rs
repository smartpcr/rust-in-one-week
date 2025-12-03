//! Windows Service support for the API server
//!
//! This module provides Windows Service functionality using the windows-service crate.

#[cfg(windows)]
pub mod windows_service {
    use std::ffi::OsString;
    use std::sync::mpsc;
    use std::time::Duration;

    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    use crate::config::Config;

    const SERVICE_NAME: &str = "WinInfraApi";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    /// Run the application as a Windows service
    pub fn run_as_service() -> Result<(), windows_service::Error> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    define_windows_service!(ffi_service_main, service_main);

    fn service_main(_arguments: Vec<OsString>) {
        if let Err(e) = run_service() {
            tracing::error!("Service error: {}", e);
        }
    }

    fn run_service() -> Result<(), Box<dyn std::error::Error>> {
        // Create a channel to receive stop events
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        // Define the service control handler
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop => {
                    let _ = shutdown_tx.send(());
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        // Register the service control handler
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Report running status
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        // Load configuration
        let config_path = get_config_path();
        let config = Config::load(&config_path).unwrap_or_default();

        // Initialize logging
        crate::init_tracing(&config.logging.level);

        tracing::info!("Service starting with config from: {:?}", config_path);
        tracing::info!(
            "Server will listen on {}:{}",
            config.server.host,
            config.server.port
        );

        // Create and run the tokio runtime
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async {
            let state = std::sync::Arc::new(crate::AppState);
            let app = crate::create_router(state);

            let addr: std::net::SocketAddr = config.socket_addr().parse()?;
            let listener = tokio::net::TcpListener::bind(addr).await?;

            tracing::info!("API server listening on {}", addr);

            // Spawn a task to handle shutdown
            let server = axum::serve(listener, app);

            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        tracing::error!("Server error: {}", e);
                    }
                }
                _ = tokio::task::spawn_blocking(move || {
                    // Wait for stop signal
                    let _ = shutdown_rx.recv();
                }) => {
                    tracing::info!("Received stop signal");
                }
            }

            Ok::<(), Box<dyn std::error::Error>>(())
        })?;

        // Report stopped status
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }

    /// Get the configuration file path
    ///
    /// When running as a service, looks for config.toml in:
    /// 1. Same directory as the executable
    /// 2. Falls back to current directory
    fn get_config_path() -> std::path::PathBuf {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let config_path = exe_dir.join("config.toml");
                if config_path.exists() {
                    return config_path;
                }
            }
        }
        std::path::PathBuf::from("config.toml")
    }
}

#[cfg(not(windows))]
pub mod windows_service {
    /// Placeholder for non-Windows platforms
    pub fn run_as_service() -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows service is only supported on Windows",
        ))
    }
}
