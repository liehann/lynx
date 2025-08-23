use axum::{
    body::Body,
    http::{Request, StatusCode, header::HOST},
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::util::ServiceExt;

use lynx::*;

async fn create_test_app() -> axum::Router {
    // Load test environment
    dotenvy::from_filename(".env.test").ok();
    
    // Create test configuration from environment
    let config = config::Config::from_env().expect("Failed to load test config");

    // Create in-memory cache
    let cache = Arc::new(RwLock::new(HashMap::new()));

    // Connect to test database
    let db = database::Database::new(&config.database_url).await.expect("Failed to connect to test database");

    let state = AppState {
        db,
        cache,
        config,
    };

    create_app(state)
}

#[tokio::test]
async fn test_admin_home_page() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/")
        .header(HOST, "lynx")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_list_links() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/api/links")
        .header(HOST, "lynx")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_redirector_no_match() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/nonexistent")
        .header(HOST, "go")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should redirect to admin add page
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    
    let location = response.headers().get("location").unwrap();
    let location_str = location.to_str().unwrap();
    // Should redirect to admin add page with source prefilled
    assert!(location_str.contains("/add"));
    assert!(location_str.contains("source=%2Fnonexistent")); // URL encoded /nonexistent
}

#[tokio::test]
async fn test_redirector_logic() {
    use lynx::redirector::*;
    use lynx::models::Link;
    use chrono::Utc;
    use std::collections::HashMap;
    
    // Test parameterized matching
    let mut cache = HashMap::new();
    let link = Link {
        id: 1,
        host: "go".to_string(),
        source: "/user/{id}".to_string(),
        target: "https://example.com/profile?id={id}".to_string(),
        created_at: Utc::now(),
    };
    cache.insert(("go".to_string(), "/user/{id}".to_string()), link);
    
    // This would normally be tested via the private functions
    // For now, we'll test the public interface through HTTP requests
}

// Note: For real testing, you'd want to:
// 1. Set up a test database that resets between tests
// 2. Test all CRUD operations
// 3. Test redirect functionality with actual links
// 4. Test parameter substitution
// 5. Test uniqueness constraints
