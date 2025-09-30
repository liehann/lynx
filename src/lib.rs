pub mod config;
pub mod models;
pub mod handlers;
pub mod redirector;
pub mod database;
pub mod templates;

use axum::{
    extract::Host,
    http::Uri,
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{trace::TraceLayer, services::ServeDir, cors::CorsLayer};

use config::Config;
use models::Link;
use database::Database;

pub type LinkCache = Arc<RwLock<HashMap<(String, String), Link>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub cache: LinkCache,
    pub config: Config,
}

pub fn create_app(state: AppState) -> Router {
    // API routes
    let api_routes = Router::new()
        .route("/api/links", get(handlers::api::list_links))
        .route("/api/links", post(handlers::api::create_link))
        .route("/api/links/search", get(handlers::api::search_links))
        .route("/api/links/reverse", get(handlers::api::get_links_by_target))
        .route("/api/links/:id", get(handlers::api::get_link))
        .route("/api/links/:id", put(handlers::api::update_link))
        .route("/api/links/:id", delete(handlers::api::delete_link));

    // UI routes  
    let ui_routes = Router::new()
        .route("/", get(handlers::ui::home))
        .route("/add", get(handlers::ui::add_page))
        .route("/add", post(handlers::ui::add_link))
        .route("/edit/:id", get(handlers::ui::edit_page))
        .route("/edit/:id", post(handlers::ui::edit_link))
        .route("/delete/:id", post(handlers::ui::delete_link))
        .route("/search", get(handlers::ui::search_page));

    Router::new()
        .merge(api_routes)
        .merge(ui_routes)
        .nest_service("/static", ServeDir::new("static"))
        .fallback(main_handler)
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn main_handler(
    Host(host): Host,
    uri: Uri,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Response {
    let path = uri.path();
    
    // Route based on host and path
    if host == state.config.admin_host {
        // Admin routes are handled by the router above
        // This fallback should only catch unmatched admin routes
        handlers::not_found().await
    } else if host == state.config.default_redirect_host {
        // Handle admin routes on default hostname (e.g., go/admin)
        if path == "/admin" {
            return handlers::ui::admin_home(axum::extract::State(state)).await
                .unwrap_or_else(|(status, msg)| (status, msg).into_response());
        } else if path.starts_with("/admin/") {
            let name = &path[7..]; // Remove "/admin/" prefix
            if !name.is_empty() {
                return handlers::ui::admin_edit_by_name(
                    axum::extract::Path(name.to_string()),
                    axum::extract::State(state)
                ).await.unwrap_or_else(|(status, msg)| (status, msg).into_response());
            }
        }
        
        // Regular redirector for all other paths on default hostname
        redirector::handle_redirect(host, path.to_string(), state).await
    } else {
        // Redirector for all other hosts
        redirector::handle_redirect(host, path.to_string(), state).await
    }
}
