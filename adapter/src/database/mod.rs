use shared::config::DatabaseConfig;
use sqlx::postgres::PgConnectOptions;

fn make_pg_connect_options(cfg: &DatabaseConfig) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.username)
        .password(&cfg.password)
        .database(&cfg.database)
}

#[derive(Clone)]
pub struct ConnectionPool(sqlx::PgPool);

impl ConnectionPool {
    pub fn inner_ref(&self) -> &sqlx::PgPool {
        &self.0
    }
}

pub fn connect_database_with(cfg: &DatabaseConfig) -> ConnectionPool {
    ConnectionPool(sqlx::PgPool::connect_lazy_with(make_pg_connect_options(
        cfg,
    )))
}
