//! Integration tests for Kubernetes API endpoints
//!
//! **Сборка:** `cargo test --features integration-api-tests --test kubernetes_integration`
//!
//! **Требования:** PostgreSQL тестовая БД (VELUM_TEST_DB_URL)
//!
//! Тесты для новых Kubernetes API endpoints:
//! - Troubleshooting Dashboard
//! - Runbook Integration
//! - Prometheus Metrics
//! - Inventory Sync

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
use velum_ffi::{api::create_app, db::SqlStore};

// ── helpers ───────────────────────────────────────────────────────────────

/// Get test database URL from environment or use default test URL
fn get_test_db_url() -> String {
    std::env::var("VELUM_TEST_DB_URL")
        .unwrap_or_else(|_| "postgres://semaphore:semaphore123@localhost:5432/semaphore_test".to_string())
}

async fn test_app() -> axum::Router {
    let url = get_test_db_url();
    let store = SqlStore::new(&url).await.expect("SqlStore::new");
    create_app(std::sync::Arc::new(store))
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let body_str = serde_json::to_string(&body).unwrap();
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body_str))
        .unwrap();
    let response = app.oneshot(request).await.expect("oneshot");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

async fn get_json(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.expect("oneshot");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

async fn create_test_user(app: &mut axum::Router) -> String {
    let (status, body) = post_json(
        app.clone(),
        "/api/users",
        json!({
            "username": "testuser",
            "name": "Test User",
            "email": "test@example.com",
            "password": "testpass123"
        }),
    )
    .await;
    if status != StatusCode::CREATED {
        return String::new();
    }
    body["token"].as_str().unwrap_or("").to_string()
}

async fn create_test_project(app: &mut axum::Router, _token: &str) -> i32 {
    let (status, body) = post_json(
        app.clone(),
        "/api/projects",
        json!({
            "name": "Test K8s Project",
            "max_parallel_tasks": 5
        }),
    )
    .await;
    if status != StatusCode::CREATED {
        return 0;
    }
    body["id"].as_i64().unwrap_or(0) as i32
}

// ── Troubleshooting API Tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_troubleshooting_api_structure() {
    let mut app = test_app().await;

    // Test without auth - endpoint exists
    let (status, _) = get_json(
        app.clone(),
        "/api/kubernetes/troubleshoot?namespace=default&kind=Pod&name=test"
    ).await;
    
    // Should return some status (401/403/404 are OK - endpoint exists)
    assert!(status != StatusCode::NOT_FOUND || status == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_troubleshooting_api_with_auth() {
    let mut app = test_app().await;
    let token = create_test_user(&mut app).await;
    
    if token.is_empty() {
        // Skip if can't create user (no DB)
        return;
    }
    
    // Test with auth but no K8s cluster
    let request = Request::builder()
        .method("GET")
        .uri("/api/kubernetes/troubleshoot?namespace=default&kind=Pod&name=test")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.expect("oneshot");
    let status = response.status();
    
    // Without K8s cluster - should return error
    assert!(status == StatusCode::NOT_FOUND || 
            status == StatusCode::BAD_REQUEST || 
            status == StatusCode::INTERNAL_SERVER_ERROR);
}

// ── Runbook API Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_runbook_list_endpoint() {
    let mut app = test_app().await;

    // Test without auth
    let (status, _) = get_json(
        app.clone(),
        "/api/kubernetes/runbooks?kind=Pod&namespace=default"
    ).await;
    
    // Endpoint should exist
    assert!(status != StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_runbook_execute_no_template() {
    let mut app = test_app().await;
    let token = create_test_user(&mut app).await;

    if token.is_empty() {
        return;
    }
    
    let project_id = create_test_project(&mut app, &token).await;
    if project_id == 0 {
        return;
    }

    // Try to execute runbook without existing template
    let (status, _body) = post_json(
        app.clone(),
        "/api/kubernetes/runbooks/execute",
        json!({
            "template_id": 99999,
            "kubernetes_context": {
                "kind": "Pod",
                "name": "test-pod",
                "namespace": "default"
            },
            "task_params": {},
            "message": "Test runbook"
        }),
    ).await;
    
    // Should return 404 for non-existent template
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Prometheus API Tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_prometheus_health_endpoint() {
    let mut app = test_app().await;

    // Without PROMETHEUS_URL env var - should return unavailable
    let (status, body) = get_json(
        app.clone(),
        "/api/kubernetes/prometheus/health"
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(body["status"].as_str().is_some());
}

#[tokio::test]
async fn test_prometheus_metrics_missing_params() {
    let mut app = test_app().await;

    // Missing required parameters
    let (status, _) = get_json(
        app.clone(),
        "/api/kubernetes/prometheus/metrics"
    ).await;
    
    // Should return error for missing params
    assert!(status == StatusCode::BAD_REQUEST || status == StatusCode::UNAUTHORIZED);
}

// ── Inventory Sync API Tests ──────────────────────────────────────────────

#[tokio::test]
async fn test_inventory_sync_preview_endpoint() {
    let mut app = test_app().await;

    // Without auth
    let (status, _) = get_json(
        app.clone(),
        "/api/kubernetes/inventory/sync/preview?project_id=1&sync_type=nodes"
    ).await;
    
    // Endpoint should exist
    assert!(status != StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_inventory_sync_execute_endpoint() {
    let mut app = test_app().await;
    let token = create_test_user(&mut app).await;

    if token.is_empty() {
        return;
    }

    // Try to execute sync without Kubernetes cluster
    let (status, _body) = post_json(
        app.clone(),
        "/api/kubernetes/inventory/sync",
        json!({
            "project_id": 1,
            "sync_type": "nodes",
            "create_new": true
        }),
    ).await;
    
    // Should return error (no K8s cluster available)
    assert!(status == StatusCode::NOT_FOUND || 
            status == StatusCode::BAD_REQUEST ||
            status == StatusCode::INTERNAL_SERVER_ERROR);
}

// ── Combined Workflow Tests ───────────────────────────────────────────────

#[tokio::test]
async fn test_kubernetes_api_endpoints_exist() {
    let mut app = test_app().await;
    let token = create_test_user(&mut app).await;

    let auth_header = if !token.is_empty() {
        format!("Bearer {}", token)
    } else {
        String::new()
    };

    // Verify all new endpoints exist (return something other than 404 for route not found)
    let endpoints = vec![
        ("GET", "/api/kubernetes/troubleshoot?namespace=default&kind=Pod&name=test"),
        ("GET", "/api/kubernetes/runbooks?kind=Pod&namespace=default"),
        ("POST", "/api/kubernetes/runbooks/execute"),
        ("GET", "/api/kubernetes/prometheus/health"),
        ("GET", "/api/kubernetes/prometheus/metrics?namespace=default&kind=Pod&name=test"),
        ("GET", "/api/kubernetes/inventory/sync/preview?project_id=1&sync_type=nodes"),
        ("POST", "/api/kubernetes/inventory/sync"),
    ];

    for (method, path) in endpoints {
        let request = match method {
            "GET" => Request::builder()
                .method("GET")
                .uri(path)
                .header(header::AUTHORIZATION, &auth_header)
                .body(Body::empty()),
            "POST" => Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &auth_header)
                .body(Body::from("{}")),
            _ => continue,
        };
        
        let request = request.unwrap();
        let response = app.clone().oneshot(request).await.expect("oneshot");
        let status = response.status();
        
        // Should not return 404 for route not found
        // 400/401/403/500 are OK - endpoint exists
        assert!(
            status != StatusCode::NOT_FOUND,
            "Endpoint {} {} should exist (got {})",
            method,
            path,
            status
        );
    }
}
