use axum::{
    response::{IntoResponse, Redirect, Response},
};
use std::collections::HashMap;

use crate::{AppState, models::Link};

pub async fn handle_redirect(host: String, path: String, state: AppState) -> Response {
    // Try to find a matching link in the cache
    let cache = state.cache.read().await;
    
    // First try exact match
    if let Some(link) = cache.get(&(host.clone(), path.clone())) {
        return create_redirect_response(&link.target, &path);
    }
    
    // Try progressive path splitting
    if let Some(target) = find_matching_rule(&cache, &host, &path) {
        return create_redirect_response(&target, &path);
    }
    
    // No match found, redirect to admin add page with source prefilled
    let admin_url = format!("http://{}:3000/add?source={}", 
                           state.config.admin_host, 
                           urlencoding::encode(&path));
    
    Redirect::temporary(&admin_url).into_response()
}

fn find_matching_rule(
    cache: &HashMap<(String, String), Link>,
    host: &str,
    path: &str,
) -> Option<String> {
    // Try parameterized rules first
    for ((cache_host, cache_source), link) in cache.iter() {
        if cache_host == host && is_parameterized_match(cache_source, path) {
            return Some(substitute_parameters(cache_source, &link.target, path));
        }
    }
    
    // Try progressive path splitting
    let separators = ['/', '.', '?'];
    let mut current_path = path.to_string();
    
    while !current_path.is_empty() {
        if let Some(link) = cache.get(&(host.to_string(), current_path.clone())) {
            return Some(link.target.clone());
        }
        
        // Split from the right using separators
        let mut found_separator = false;
        for sep in separators.iter() {
            if let Some(pos) = current_path.rfind(*sep) {
                current_path = current_path[..pos].to_string();
                found_separator = true;
                break;
            }
        }
        
        if !found_separator {
            break;
        }
    }
    
    None
}

fn is_parameterized_match(pattern: &str, path: &str) -> bool {
    // Simple parameterized matching: pattern ends with {param}
    if let Some(param_start) = pattern.rfind("/{") {
        if pattern.ends_with("}") {
            let prefix = &pattern[..param_start + 1];
            return path.starts_with(prefix) && path.len() > prefix.len();
        }
    }
    false
}

fn substitute_parameters(pattern: &str, target: &str, path: &str) -> String {
    if let Some(param_start) = pattern.rfind("/{") {
        if pattern.ends_with("}") {
            let prefix = &pattern[..param_start + 1];
            if path.starts_with(prefix) {
                let param_value = &path[prefix.len()..];
                // Extract parameter name from pattern
                let param_name = &pattern[param_start + 2..pattern.len() - 1];
                let param_placeholder = format!("{{{}}}", param_name);
                return target.replace(&param_placeholder, param_value);
            }
        }
    }
    target.to_string()
}

fn create_redirect_response(target: &str, _path: &str) -> Response {
    Redirect::temporary(target).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Link;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_link(id: i32, host: &str, source: &str, target: &str) -> Link {
        Link {
            id,
            host: host.to_string(),
            source: source.to_string(),
            target: target.to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_exact_match() {
        let mut cache = HashMap::new();
        let link = create_test_link(1, "go", "/test", "https://example.com");
        cache.insert(("go".to_string(), "/test".to_string()), link);
        
        let result = find_matching_rule(&cache, "go", "/test");
        assert_eq!(result, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_parameterized_match() {
        let mut cache = HashMap::new();
        let link = create_test_link(1, "go", "/user/{id}", "https://example.com/profile?id={id}");
        cache.insert(("go".to_string(), "/user/{id}".to_string()), link);
        
        let result = find_matching_rule(&cache, "go", "/user/123");
        assert_eq!(result, Some("https://example.com/profile?id=123".to_string()));
    }

    #[test]
    fn test_progressive_path_splitting() {
        let mut cache = HashMap::new();
        let link = create_test_link(1, "go", "/docs", "https://example.com/documentation");
        cache.insert(("go".to_string(), "/docs".to_string()), link);
        
        let result = find_matching_rule(&cache, "go", "/docs/api/v1");
        assert_eq!(result, Some("https://example.com/documentation".to_string()));
    }

    #[test]
    fn test_is_parameterized_match() {
        assert!(is_parameterized_match("/user/{id}", "/user/123"));
        assert!(is_parameterized_match("/api/v1/{endpoint}", "/api/v1/users"));
        assert!(!is_parameterized_match("/user/static", "/user/123"));
        assert!(!is_parameterized_match("/user/{id", "/user/123"));
    }

    #[test]
    fn test_substitute_parameters() {
        let result = substitute_parameters("/user/{id}", "https://example.com/profile?id={id}", "/user/123");
        assert_eq!(result, "https://example.com/profile?id=123");
        
        let result = substitute_parameters("/api/{version}", "https://api.example.com/{version}/data", "/api/v2");
        assert_eq!(result, "https://api.example.com/v2/data");
    }
}
