use std::net::{Ipv4Addr, SocketAddr};

use adapter::database::connect_database_with;
use api::route::health::build_health_check_routers;
use shared::config::AppConfig;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap().await
}

async fn bootstrap() -> anyhow::Result<()> {
    let app_config = AppConfig::new()?;
    let pool = connect_database_with(&app_config.database);
    let registry = registry::AppRegistry::new(pool);
    let app = axum::Router::new()
        .merge(build_health_check_routers())
        .with_state(registry);

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8081);
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {addr}");

    Ok(axum::serve(listener, app).await?)
}
