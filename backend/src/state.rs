use std::sync::Arc;

use sqlx::SqlitePool;

use crate::config::Config;
use crate::provider::ChatProvider;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub provider: Arc<dyn ChatProvider>,
    pub config: Config,
}

impl AppState {
    pub fn new(pool: SqlitePool, provider: Arc<dyn ChatProvider>, config: Config) -> Self {
        Self {
            pool,
            provider,
            config,
        }
    }
}
