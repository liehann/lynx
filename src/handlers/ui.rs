use axum::{
    extract::{Path, Query, State, Form},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    models::{CreateLinkRequest, UpdateLinkRequest, LinkResponse, SearchQuery},
    templates::{HomeTemplate, AddTemplate, EditTemplate, SearchTemplate},
    AppState,
};

#[derive(Deserialize)]
pub struct AddFormData {
    pub host: String,
    pub source: String,
    pub target: String,
}

#[derive(Deserialize)]
pub struct EditFormData {
    pub host: String,
    pub source: String,
    pub target: String,
}

#[derive(Deserialize)]
pub struct AddPageQuery {
    pub source: Option<String>,
}

pub async fn home(State(state): State<AppState>) -> Result<Response, (StatusCode, String)> {
    match state.db.get_recent_links(20).await {
        Ok(links) => {
            let responses: Vec<LinkResponse> = links.into_iter().map(LinkResponse::from).collect();
            let template = HomeTemplate { links: responses };
            Ok(template.into_response())
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load links: {}", e),
        )),
    }
}

pub async fn add_page(
    Query(params): Query<AddPageQuery>,
    State(state): State<AppState>,
) -> Response {
    let source = params.source.as_deref().unwrap_or("");
    let template = AddTemplate { 
        source, 
        error: None,
        default_redirect_host: &state.config.default_redirect_host,
    };
    template.into_response()
}

pub async fn add_link(
    State(state): State<AppState>,
    Form(form_data): Form<AddFormData>,
) -> Response {
    let request = CreateLinkRequest {
        host: form_data.host.clone(),
        source: form_data.source.clone(),
        target: form_data.target,
    };

    // Check for conflicts
    match state.db.check_source_conflict(&request.host, &request.source, None).await {
        Ok(true) => {
            let template = AddTemplate {
                source: &form_data.source,
                error: Some("A link with this host and source already exists"),
                default_redirect_host: &state.config.default_redirect_host,
            };
            return template.into_response();
        }
        Err(_e) => {
            let template = AddTemplate {
                source: &form_data.source,
                error: Some("Failed to check for conflicts"),
                default_redirect_host: &state.config.default_redirect_host,
            };
            return template.into_response();
        }
        Ok(false) => {}
    }

    match state.db.create_link(&request).await {
        Ok(link) => {
            // Update cache
            let mut cache = state.cache.write().await;
            cache.insert((link.host.clone(), link.source.clone()), link);
            
            Redirect::to("/").into_response()
        }
        Err(e) => {
            let error_msg = if e.to_string().contains("duplicate key") || e.to_string().contains("unique constraint") {
                "A link with this host and source already exists"
            } else {
                "Failed to create link"
            };
            
            let template = AddTemplate {
                source: &form_data.source,
                error: Some(error_msg),
                default_redirect_host: &state.config.default_redirect_host,
            };
            template.into_response()
        }
    }
}

pub async fn edit_page(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Response, (StatusCode, String)> {
    match state.db.get_link_by_id(id).await {
        Ok(Some(link)) => {
            let response = LinkResponse::from(link);
            let template = EditTemplate { 
                link: &response, 
                error: None 
            };
            Ok(template.into_response())
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, "Link not found".to_string())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load link: {}", e),
        )),
    }
}

pub async fn edit_link(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Form(form_data): Form<EditFormData>,
) -> Response {
    // Get the existing link
    let existing = match state.db.get_link_by_id(id).await {
        Ok(Some(link)) => link,
        Ok(None) => return (StatusCode::NOT_FOUND, "Link not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load link").into_response(),
    };

    let request = UpdateLinkRequest {
        host: Some(form_data.host.clone()),
        source: Some(form_data.source.clone()),
        target: Some(form_data.target),
    };

    // Check for conflicts if host or source changed
    if form_data.host != existing.host || form_data.source != existing.source {
        match state.db.check_source_conflict(&form_data.host, &form_data.source, Some(id)).await {
            Ok(true) => {
                let response = LinkResponse::from(existing);
                let template = EditTemplate {
                    link: &response,
                    error: Some("A link with this host and source already exists"),
                };
                return template.into_response();
            }
            Err(_) => {
                let response = LinkResponse::from(existing);
                let template = EditTemplate {
                    link: &response,
                    error: Some("Failed to check for conflicts"),
                };
                return template.into_response();
            }
            Ok(false) => {}
        }
    }

    match state.db.update_link(id, &request).await {
        Ok(Some(updated_link)) => {
            // Update cache
            let mut cache = state.cache.write().await;
            cache.remove(&(existing.host, existing.source));
            cache.insert((updated_link.host.clone(), updated_link.source.clone()), updated_link);
            
            Redirect::to("/").into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Link not found").into_response(),
        Err(e) => {
            let error_msg = if e.to_string().contains("duplicate key") || e.to_string().contains("unique constraint") {
                "A link with this host and source already exists"
            } else {
                "Failed to update link"
            };
            
            let response = LinkResponse::from(existing);
            let template = EditTemplate {
                link: &response,
                error: Some(error_msg),
            };
            template.into_response()
        }
    }
}

pub async fn search_page(
    Query(params): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Result<Response, (StatusCode, String)> {
    let query = params.q.as_deref().unwrap_or("");
    let page = params.page.unwrap_or(1).max(1);
    let per_page = 20;

    let links = if query.is_empty() {
        Vec::new()
    } else {
        match state.db.search_links(query, page, per_page).await {
            Ok(links) => links.into_iter().map(LinkResponse::from).collect(),
            Err(e) => return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to search links: {}", e),
            )),
        }
    };

    let template = SearchTemplate {
        query,
        links,
        page,
    };
    
    Ok(template.into_response())
}

pub async fn delete_link(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Response {
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
            
            Redirect::to("/").into_response()
        }
        Ok(false) => (StatusCode::NOT_FOUND, "Link not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete link").into_response(),
    }
}
