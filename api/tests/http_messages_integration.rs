use axum::{body::Body, http::{Request, StatusCode}, routing::{get, post, put, delete}, Router};
use tower::util::ServiceExt;
use tower_http::add_extension::AddExtensionLayer;
use communities_core::create_repositories;
use communities_core::domain::message::ports::MessageRepository;
use uuid::Uuid;
use serde_json::json;
use api as crate_api;
use crate_api::http::messages::handlers as handlers;
use crate_api::http::server::app_state::AppState;
use crate_api::http::server::middleware::auth::entities::UserIdentity;

// Helper: start docker mongo if MONGO_TEST_URI not set
async fn ensure_mongo_uri() -> Option<(String, Option<String>)> {
    let env_uri = std::env::var("MONGO_TEST_URI").ok();
    let db_name = std::env::var("MONGO_TEST_DB").unwrap_or_else(|_| "message_test_db".into());

    if let Some(u) = env_uri {
        return Some((u, Some(db_name)));
    }

    // Try to start docker container
    use std::process::Command;
    let docker_check = Command::new("docker").arg("version").output();
    if docker_check.is_err() {
        return None;
    }

    let run = Command::new("docker")
        .args(["run", "-d", "-P", "--rm", "mongo:6.0"])
        .output()
        .ok()?;
    if !run.status.success() {
        return None;
    }
    let container_id = String::from_utf8_lossy(&run.stdout).trim().to_string();
    let port_out = Command::new("docker").args(["port", &container_id, "27017"]).output().ok()?;
    if !port_out.status.success() {
        return None;
    }
    let out = String::from_utf8_lossy(&port_out.stdout);
    let host_port = out.trim().rsplit(':').next().unwrap().to_string();
    let uri = format!("mongodb://127.0.0.1:{}", host_port);
    // wait for readiness
    // wait for mongo to accept connections by retrying create_repositories
    for _ in 0..40 {
        if create_repositories(&uri, &db_name).await.is_ok() {
            return Some((uri, Some(container_id)));
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    let _ = Command::new("docker").args(["rm", "-f", &container_id]).output();
    None
}

#[tokio::test]
async fn http_handlers_crud_flow() {
    // ensure mongo
    let maybe = ensure_mongo_uri().await;
    let (uri, container_id_opt) = match maybe {
        Some((u, cid)) => (u, cid),
        None => {
            eprintln!("Skipping API integration test: no Mongo available and docker not present");
            return;
        }
    };

    // create repositories
    let repos = create_repositories(&uri, "message_test_db").await.expect("create repos");
    let state: AppState = repos.clone().into();

    // prepare router with extension providing UserIdentity
    let user_id = Uuid::new_v4();
    let user_identity = UserIdentity { user_id };

    let router = Router::new()
        .route("/messages", post(handlers::create_message))
        .route("/messages/{id}", get(handlers::get_message))
        .route("/messages", get(handlers::list_messages))
        .route("/messages/{id}", put(handlers::update_message))
        .route("/messages/{id}", delete(handlers::delete_message))
        .with_state(state.clone())
        .layer(AddExtensionLayer::new(user_identity.clone()));

    // create message
    let channel = Uuid::new_v4();
    let req_body = json!({
        "channel_id": channel,
        "content": "integration via http",
        "reply_to_message_id": null,
        "attachments": []
    });

    let request = Request::builder()
        .method("POST")
        .uri("/messages")
        .header("content-type", "application/json")
        .body(Body::from(req_body.to_string()))
        .unwrap();

    let response = router.clone().oneshot(request).await.expect("router oneshot");
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify insertion via the repository and obtain the id
    use communities_core::domain::common::GetPaginated;
    let (messages, _total) = repos.message_repository.list(&GetPaginated::default()).await.expect("list messages");
    assert!(!messages.is_empty());
    let id = messages[0].id.0;
    let request = Request::builder()
        .method("GET")
        .uri(format!("/messages/{}", id))
        .body(Body::empty())
        .unwrap();
    let response = router.clone().oneshot(request).await.expect("get oneshot");
    assert_eq!(response.status(), StatusCode::OK);

    // cleanup docker container if we started one
    if let Some(cid) = container_id_opt {
        let _ = std::process::Command::new("docker").args(["rm", "-f", &cid]).output();
    }
}
