use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;

use crate::{
    models::{CreateLinkRequest, UpdateLinkRequest, LinkResponse, ErrorResponse, SearchQuery},
    AppState,
};

pub async fn list_links(State(state): State<AppState>) -> Result<Json<Vec<LinkResponse>>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_recent_links(50).await {
        Ok(links) => {
            let responses: Vec<LinkResponse> = links.into_iter().map(LinkResponse::from).collect();
            Ok(Json(responses))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch links: {}", e),
            }),
        )),
    }
}

pub async fn get_link(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<LinkResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_link_by_id(id).await {
        Ok(Some(link)) => Ok(Json(LinkResponse::from(link))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Link not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch link: {}", e),
            }),
        )),
    }
}

pub async fn create_link(
    State(state): State<AppState>,
    Json(request): Json<CreateLinkRequest>,
) -> Result<Json<LinkResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check for conflicts
    if let Ok(has_conflict) = state.db.check_source_conflict(&request.host, &request.source, None).await {
        if has_conflict {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: format!("A link with host '{}' and source '{}' already exists", request.host, request.source),
                }),
            ));
        }
    }

    match state.db.create_link(&request).await {
        Ok(link) => {
            // Update cache
            let mut cache = state.cache.write().await;
            cache.insert((link.host.clone(), link.source.clone()), link.clone());
            
            Ok(Json(LinkResponse::from(link)))
        }
        Err(e) => {
            // Handle unique constraint violation
            if e.to_string().contains("duplicate key") || e.to_string().contains("unique constraint") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: format!("A link with host '{}' and source '{}' already exists", request.host, request.source),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to create link: {}", e),
                    }),
                ))
            }
        }
    }
}

pub async fn update_link(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(request): Json<UpdateLinkRequest>,
) -> Result<Json<LinkResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get the existing link to check current values
    let existing = match state.db.get_link_by_id(id).await {
        Ok(Some(link)) => link,
        Ok(None) => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Link not found".to_string(),
            }),
        )),
        Err(e) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch link: {}", e),
            }),
        )),
    };

    // Check for conflicts if host or source is being changed
    let new_host = request.host.as_ref().unwrap_or(&existing.host);
    let new_source = request.source.as_ref().unwrap_or(&existing.source);
    
    if new_host != &existing.host || new_source != &existing.source {
        if let Ok(has_conflict) = state.db.check_source_conflict(new_host, new_source, Some(id)).await {
            if has_conflict {
                return Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: format!("A link with host '{}' and source '{}' already exists", new_host, new_source),
                    }),
                ));
            }
        }
    }

    match state.db.update_link(id, &request).await {
        Ok(Some(updated_link)) => {
            // Update cache - remove old entry and add new one
            let mut cache = state.cache.write().await;
            cache.remove(&(existing.host, existing.source));
            cache.insert((updated_link.host.clone(), updated_link.source.clone()), updated_link.clone());
            
            Ok(Json(LinkResponse::from(updated_link)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Link not found".to_string(),
            }),
        )),
        Err(e) => {
            // Handle unique constraint violation
            if e.to_string().contains("duplicate key") || e.to_string().contains("unique constraint") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: format!("A link with host '{}' and source '{}' already exists", new_host, new_source),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to update link: {}", e),
                    }),
                ))
            }
        }
    }
}

pub async fn delete_link(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    // Get the link first to remove from cache
    let existing = match state.db.get_link_by_id(id).await {
        Ok(Some(link)) => Some(link),
        Ok(None) => None,
        Err(_) => None,
    };

    match state.db.delete_link(id).await {
        Ok(true) => {
            // Remove from cache if we found the existing link
            if let Some(link) = existing {
                let mut cache = state.cache.write().await;
                cache.remove(&(link.host, link.source));
            }
            
            Ok(Json(serde_json::json!({"message": "Link deleted successfully"})))
        }
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Link not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to delete link: {}", e),
            }),
        )),
    }
}

pub async fn search_links(
    Query(params): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<LinkResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let query = params.q.unwrap_or_default();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100).max(1);

    match state.db.search_links(&query, page, per_page).await {
        Ok(links) => {
            let responses: Vec<LinkResponse> = links.into_iter().map(LinkResponse::from).collect();
            Ok(Json(responses))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to search links: {}", e),
            }),
        )),
    }
}

pub async fn get_links_by_target(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Vec<LinkResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let target = match params.get("target") {
        Some(t) => t,
        None => return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing 'target' query parameter".to_string(),
            }),
        )),
    };

    match state.db.get_links_by_target(target).await {
        Ok(links) => {
            let responses: Vec<LinkResponse> = links.into_iter().map(LinkResponse::from).collect();
            Ok(Json(responses))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch links by target: {}", e),
            }),
        )),
    }
}
