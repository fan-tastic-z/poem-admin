use error_stack::{Result, ResultExt};
use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{Error, config::config::Config};

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Pool<Postgres>,
}

impl Db {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let opts = PgConnectOptions::new()
            .host(&config.database.host)
            .port(config.database.port)
            .username(&config.database.username)
            .password(&config.database.password)
            .database(&config.database.database_name);
        let pool = PgPoolOptions::new()
            .connect_with(opts)
            .await
            .change_context_lazy(|| Error("failed to connect to database".to_string()))?;
        Ok(Self { pool })
    }
}
