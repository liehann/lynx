use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: i32,
    pub host: String,
    pub source: String,
    pub target: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub host: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkRequest {
    pub host: Option<String>,
    pub source: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub id: i32,
    pub host: String,
    pub source: String,
    pub target: String,
    pub created_at: DateTime<Utc>,
}

impl From<Link> for LinkResponse {
    fn from(link: Link) -> Self {
        Self {
            id: link.id,
            host: link.host,
            source: link.source,
            target: link.target,
            created_at: link.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}
