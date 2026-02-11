use crate::types::config::AppConfig;
use reqwest::Client;
use rusqlite::Connection;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub config: Mutex<AppConfig>,
    pub http_client: Client,
    pub queue_paused: AtomicBool,
    pub pipeline_cancelled: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(conn: Connection, config: AppConfig) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            db: Mutex::new(conn),
            config: Mutex::new(config),
            http_client,
            queue_paused: AtomicBool::new(false),
            pipeline_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}
