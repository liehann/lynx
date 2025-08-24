use lynx::*;
use lynx::config::Config;
use lynx::database::Database;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    // Initialize database
    let db = Database::new(&config.database_url).await?;

    // Initialize cache
    let cache = Arc::new(RwLock::new(HashMap::new()));
    
    // Load links into cache
    let links = db.get_all_links().await?;
    {
        let mut cache_write = cache.write().await;
        for link in links {
            cache_write.insert((link.host.clone(), link.source.clone()), link);
        }
    }

    let state = AppState {
        db,
        cache,
        config: config.clone(),
    };

    // Build our application with routes
    let app = create_app(state);

    let bind_addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    
    println!("Server running on http://{}", bind_addr);
    println!("Admin UI: http://{}:{}", config.admin_host, config.port);
    println!("Redirector: http://{}:{}", config.default_redirect_host, config.port);

    axum::serve(listener, app).await?;

    Ok(())
}


