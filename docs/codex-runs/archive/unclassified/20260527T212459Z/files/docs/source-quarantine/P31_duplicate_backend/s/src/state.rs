use crate::types::config::AppConfig;
use reqwest::Client;
use rusqlite::Connection;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use tokio::sync::broadcast;

/// Global app state shared across Tauri commands.
///
/// **Lock ordering convention:** When both locks are needed in the same scope,
/// always acquire `config` (read or write) BEFORE `db`. This prevents deadlocks.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub config: RwLock<AppConfig>,
    pub http_client: Client,
    pub queue_paused: AtomicBool,
    pub pipeline_cancelled: Arc<AtomicBool>,
    pub shutdown_tx: broadcast::Sender<()>,
}

impl AppState {
    pub fn new(conn: Connection, config: AppConfig) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .expect("Failed to build HTTP client");

        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            db: Mutex::new(conn),
            config: RwLock::new(config),
            http_client,
            queue_paused: AtomicBool::new(false),
            pipeline_cancelled: Arc::new(AtomicBool::new(false)),
            shutdown_tx,
        }
    }
}
