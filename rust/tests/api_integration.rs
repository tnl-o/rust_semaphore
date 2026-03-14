//! Integration tests for the Semaphore REST API
//!
//! Tests run against a real SQLite in-memory-like (temp-file) database and
//! the actual Axum router.  No mocks, no stubs — every test hits the full
//! HTTP stack, exactly like a real client would.
//!
//! Pattern:
//!   1. Spin up `create_app(SqlStore)` backed by a fresh temp-file DB.
//!   2. Send HTTP requests via `tower::ServiceExt::oneshot`.
//!   3. Assert status codes and JSON payloads.
//!
//! Run with:
//!   cargo test --test api_integration

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt; // .collect() on response body
use serde_json::{json, Value};
use semaphore_ffi::{api::create_app, db::SqlStore};
use tower::ServiceExt; // .oneshot()

// ── helpers ───────────────────────────────────────────────────────────────

/// Convert a Windows path (possibly with \\?\ prefix) to a sqlite:// URL.
fn sqlite_url_from_path(path: &std::path::Path) -> String {
    // canonicalize() resolves the \\?\ extended-length prefix on Windows
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    // Strip \\?\ or \\?\UNC\ prefixes that Windows sometimes adds
    let path_str = canonical.to_string_lossy();
    let stripped = path_str
        .strip_prefix(r"\\?\UNC\")
        .map(|s| format!("//{}",  s))
        .or_else(|| path_str.strip_prefix(r"\\?\").map(|s| s.to_string()))
        .unwrap_or_else(|| path_str.into_owned());
    // Forward slashes
    let forward = stripped.replace('\\', "/");
    format!("sqlite:///{}", forward)
}

/// Create a fresh Axum app backed by a brand-new temp-file SQLite database.
/// The `NamedTempFile` is returned so it is kept alive for the duration of
/// the test.  When it drops, the temp file is deleted automatically.
async fn test_app() -> (
    axum::Router,
    tempfile::NamedTempFile,
) {
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let url = sqlite_url_from_path(temp.path());

    let store = SqlStore::new(&url).await.expect("SqlStore::new");
    let app = create_app(Box::new(store));

    (app, temp)
}

/// POST JSON body, return (status, parsed JSON value).
async fn post_json(
    app: axum::Router,
    uri: &str,
    body: Value,
) -> (StatusCode, Value) {
    post_json_with_token(app, uri, body, None).await
}

/// POST JSON body with optional Bearer token.
async fn post_json_with_token(
    app: axum::Router,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let body_str = serde_json::to_string(&body).unwrap();
    let mut req = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(tok) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {}", tok));
    }

    let request = req.body(Body::from(body_str)).unwrap();
    let response = app.oneshot(request).await.expect("oneshot");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// GET with optional Bearer token.
async fn get_json(
    app: axum::Router,
    uri: &str,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method("GET").uri(uri);

    if let Some(tok) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {}", tok));
    }

    let request = req.body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.expect("oneshot");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// Register an admin user and return the access token.
async fn register_and_login(app: &axum::Router) -> String {
    // Register via CLI would require spawning a process; instead we create a
    // user directly through the store.  Because `create_app` takes ownership
    // of the store, we log in as the user that was pre-seeded in the DB.
    //
    // For integration tests we rely on the admin user being creatable through
    // the POST /api/users endpoint (which some semaphore builds expose) or we
    // use a dedicated test-only setup helper.
    //
    // Here we use the login endpoint — but first we need a user in the DB.
    // We call an internal helper instead of going through HTTP to avoid
    // circular dependency.
    //
    // Strategy: create a user through the DB store directly before calling
    // create_app.  Since we can't do that here (app already built), we use
    // the same temp DB and build a second, ephemeral store just for seeding.
    //
    // Simpler approach for integration tests:
    // We just test the happy-path of login assuming the seeding has been done.
    // The `seed_test_db` helper below creates the user before building the app.
    //
    // This function is only valid when the app was created via `seeded_app()`.
    let (status, body) = post_json(
        app.clone(),
        "/api/auth/login",
        json!({ "username": "testadmin", "password": "Test1234!" }),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "login should succeed; body={:?}",
        body
    );

    body["token"]
        .as_str()
        .expect("token in login response")
        .to_string()
}

/// Create an app with one pre-seeded admin user `testadmin / Test1234!`.
async fn seeded_app() -> (axum::Router, tempfile::NamedTempFile) {
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let url = sqlite_url_from_path(temp.path());

    // Seed the user into the DB before building the app
    {
        use semaphore_ffi::db::store::UserManager;
        use semaphore_ffi::models::User;
        use chrono::Utc;

        let store = SqlStore::new(&url).await.expect("SqlStore::new for seeding");

        let user = User {
            id: 0,
            username: "testadmin".into(),
            name: "Test Admin".into(),
            email: "testadmin@test.local".into(),
            password: String::new(), // password set via create_user second arg
            admin: true,
            external: false,
            alert: false,
            pro: false,
            created: Utc::now(),
            totp: None,
            email_otp: None,
        };

        store.create_user(user, "Test1234!").await.expect("create test user");
    }

    let store = SqlStore::new(&url).await.expect("SqlStore::new for app");
    let app = create_app(Box::new(store));

    (app, temp)
}

// ── tests ─────────────────────────────────────────────────────────────────

// ── Health check ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_check() {
    let (app, _temp) = test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Auth: login / logout / refresh ────────────────────────────────────────

#[tokio::test]
async fn test_login_wrong_password_returns_401() {
    let (app, _temp) = seeded_app().await;

    let (status, body) = post_json(
        app,
        "/api/auth/login",
        json!({ "username": "testadmin", "password": "wrongpassword" }),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "body={:?}", body);
}

#[tokio::test]
async fn test_login_unknown_user_returns_401() {
    let (app, _temp) = seeded_app().await;

    let (status, body) = post_json(
        app,
        "/api/auth/login",
        json!({ "username": "nobody", "password": "whatever" }),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "body={:?}", body);
}

#[tokio::test]
async fn test_login_success_returns_token() {
    let (app, _temp) = seeded_app().await;

    let (status, body) = post_json(
        app,
        "/api/auth/login",
        json!({ "username": "testadmin", "password": "Test1234!" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body={:?}", body);
    assert!(
        body["token"].is_string(),
        "response must contain 'token'; body={:?}",
        body
    );
}

#[tokio::test]
async fn test_refresh_token() {
    let (app, _temp) = seeded_app().await;

    // Login first
    let (_, login_body) = post_json(
        app.clone(),
        "/api/auth/login",
        json!({ "username": "testadmin", "password": "Test1234!" }),
    )
    .await;

    let refresh_token = login_body["refresh_token"]
        .as_str()
        .expect("refresh_token in login response");

    // Use refresh token
    let (status, body) = post_json(
        app,
        "/api/auth/refresh",
        json!({ "refresh_token": refresh_token }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "refresh should succeed; body={:?}", body);
    assert!(body["token"].is_string(), "new token in refresh response; body={:?}", body);
}

// ── Auth: protected endpoints require token ────────────────────────────────

#[tokio::test]
async fn test_get_projects_requires_auth() {
    let (app, _temp) = test_app().await;

    let (status, _body) = get_json(app, "/api/projects", None).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "unauthenticated request must return 401"
    );
}

#[tokio::test]
async fn test_get_current_user_requires_auth() {
    let (app, _temp) = test_app().await;

    let (status, _body) = get_json(app, "/api/user", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Projects CRUD ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_projects() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    // Create project
    let (status, body) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({
            "name": "Integration Test Project",
            "max_parallel_tasks": 1,
            "alert": false
        }),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "create project; body={:?}", body);
    let project_id = body["id"].as_i64().expect("project id in response");

    // List projects
    let (status, list_body) = get_json(app.clone(), "/api/projects", Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "list projects; body={:?}", list_body);

    let projects = list_body.as_array().expect("projects array");
    assert!(
        projects.iter().any(|p| p["id"].as_i64() == Some(project_id)),
        "created project must appear in list"
    );
}

#[tokio::test]
async fn test_delete_project() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    // Create project
    let (status, body) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "To Delete", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body={:?}", body);
    let project_id = body["id"].as_i64().expect("project id");

    // Delete project
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/projects/{}", project_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::NO_CONTENT
            || response.status() == StatusCode::OK,
        "delete project returned {:?}",
        response.status()
    );
}

// ── Users API ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_current_user_with_token() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (status, body) = get_json(app, "/api/user", Some(&token)).await;

    assert_eq!(status, StatusCode::OK, "body={:?}", body);
    assert_eq!(
        body["username"].as_str(),
        Some("testadmin"),
        "current user username; body={:?}",
        body
    );
}
