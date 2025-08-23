use anyhow::Result;
use sqlx::{PgPool, Row};


use crate::models::{Link, CreateLinkRequest, UpdateLinkRequest};

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub async fn get_all_links(&self) -> Result<Vec<Link>> {
        let rows = sqlx::query("SELECT id, host, source, target, created_at FROM links ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        
        let mut links = Vec::new();
        for row in rows {
            links.push(Link {
                id: row.get("id"),
                host: row.get("host"),
                source: row.get("source"),
                target: row.get("target"),
                created_at: row.get("created_at"),
            });
        }
        
        Ok(links)
    }

    pub async fn get_link_by_id(&self, id: i32) -> Result<Option<Link>> {
        let row = sqlx::query("SELECT id, host, source, target, created_at FROM links WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            Ok(Some(Link {
                id: row.get("id"),
                host: row.get("host"),
                source: row.get("source"),
                target: row.get("target"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn create_link(&self, request: &CreateLinkRequest) -> Result<Link> {
        let row = sqlx::query("INSERT INTO links (host, source, target) VALUES ($1, $2, $3) RETURNING id, host, source, target, created_at")
            .bind(&request.host)
            .bind(&request.source)
            .bind(&request.target)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(Link {
            id: row.get("id"),
            host: row.get("host"),
            source: row.get("source"),
            target: row.get("target"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn update_link(&self, id: i32, request: &UpdateLinkRequest) -> Result<Option<Link>> {
        // Get the existing link first
        let existing = self.get_link_by_id(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        // Use existing values if not provided in update
        let host = request.host.as_ref().unwrap_or(&existing.host);
        let source = request.source.as_ref().unwrap_or(&existing.source);
        let target = request.target.as_ref().unwrap_or(&existing.target);

        let row = sqlx::query("UPDATE links SET host = $1, source = $2, target = $3 WHERE id = $4 RETURNING id, host, source, target, created_at")
            .bind(host)
            .bind(source)
            .bind(target)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(Some(Link {
            id: row.get("id"),
            host: row.get("host"),
            source: row.get("source"),
            target: row.get("target"),
            created_at: row.get("created_at"),
        }))
    }

    pub async fn delete_link(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM links WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }

    pub async fn search_links(&self, query: &str, page: i32, per_page: i32) -> Result<Vec<Link>> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", query);
        
        let rows = sqlx::query("SELECT id, host, source, target, created_at FROM links WHERE source ILIKE $1 OR target ILIKE $1 OR host ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(search_pattern)
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;
        
        let mut links = Vec::new();
        for row in rows {
            links.push(Link {
                id: row.get("id"),
                host: row.get("host"),
                source: row.get("source"),
                target: row.get("target"),
                created_at: row.get("created_at"),
            });
        }
        
        Ok(links)
    }

    pub async fn get_recent_links(&self, limit: i32) -> Result<Vec<Link>> {
        let rows = sqlx::query("SELECT id, host, source, target, created_at FROM links ORDER BY created_at DESC LIMIT $1")
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;
        
        let mut links = Vec::new();
        for row in rows {
            links.push(Link {
                id: row.get("id"),
                host: row.get("host"),
                source: row.get("source"),
                target: row.get("target"),
                created_at: row.get("created_at"),
            });
        }
        
        Ok(links)
    }

    pub async fn check_source_conflict(&self, host: &str, source: &str, exclude_id: Option<i32>) -> Result<bool> {
        let count: i64 = if let Some(id) = exclude_id {
            let row = sqlx::query("SELECT COUNT(*) as count FROM links WHERE host = $1 AND source = $2 AND id != $3")
                .bind(host)
                .bind(source)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
            row.get("count")
        } else {
            let row = sqlx::query("SELECT COUNT(*) as count FROM links WHERE host = $1 AND source = $2")
                .bind(host)
                .bind(source)
                .fetch_one(&self.pool)
                .await?;
            row.get("count")
        };
        
        Ok(count > 0)
    }
}