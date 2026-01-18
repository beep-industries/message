use communities_core::infrastructure::message::repositories::mongo::MongoMessageRepository;
use communities_core::domain::message::ports::MessageRepository;
use communities_core::domain::message::entities::{InsertMessageInput, Attachment, AttachmentId, ChannelId, AuthorId, MessageId, UpdateMessageInput};
use communities_core::domain::common::GetPaginated;
use mongodb::{Client, options::ClientOptions};
use uuid::Uuid;

// Optional testcontainers integration to allow `cargo test` without env vars.
// If Docker is available, spin up a temporary MongoDB container.
#[tokio::test]
async fn mongo_repository_crud_flow() {
    // Try to use MONGO_TEST_URI if provided; otherwise try to start a dockerized mongo.
    let env_uri = std::env::var("MONGO_TEST_URI").ok();
    let db_name = std::env::var("MONGO_TEST_DB").unwrap_or_else(|_| "message_test_db".into());

    // connection URI we'll use
    let mut uri_to_use: Option<String> = None;

    if let Some(u) = env_uri {
        if !u.is_empty() {
            uri_to_use = Some(u);
        }
    }

    // If no env var, try to start a docker mongo using testcontainers
    // If that fails (no docker), we'll skip the test.
    // This keeps `cargo test` working locally when Docker is available.
    let mut container_to_stop: Option<String> = None;
    if uri_to_use.is_none() {
        match try_start_docker_mongo().await {
            Ok((u, cid)) => {
                uri_to_use = Some(u);
                container_to_stop = Some(cid);
            }
            Err(e) => {
                eprintln!("Skipping Mongo integration test: {}", e);
            }
        }
    }

    let uri = match uri_to_use {
        Some(u) => u,
        None => return, // skipped
    };

    let mut opts = ClientOptions::parse(&uri).await.expect("parse options");
    opts.app_name = Some("mongo_repo_integration_test".to_string());
    let client = Client::with_options(opts).expect("create client");
    let db = client.database(&db_name);

    // Wait for mongo to be ready (it may take a few seconds after container start)
    {
        use mongodb::bson::doc;
        use tokio::time::{sleep, Duration};

        let mut ready = false;
        for _ in 0..20 {
            let ping_res = db.run_command(doc! { "ping": 1 }).await;
            if ping_res.is_ok() {
                ready = true;
                break;
            }
            sleep(Duration::from_millis(250)).await;
        }
        if !ready {
            // cleanup container if started
            if let Some(cid) = container_to_stop {
                let _ = stop_docker_container(&cid);
            }
            panic!("MongoDB did not become ready in time");
        }
    }

    // ensure a clean database
    let _ = db.drop().await;

    let repo = MongoMessageRepository::new(&db);

    let id = MessageId::from(Uuid::new_v4());
    let channel = ChannelId::from(Uuid::new_v4());
    let author = AuthorId::from(Uuid::new_v4());

    let input = InsertMessageInput {
        id,
        channel_id: channel,
        author_id: author,
        content: "mongo hello".to_string(),
        reply_to_message_id: None,
        attachments: vec![Attachment { id: AttachmentId::from(Uuid::new_v4()), name: "f".into(), url: "u".into() }],
    };

    // Insert
    let inserted = repo.insert(input.clone()).await.expect("insert should succeed");
    assert_eq!(inserted.id, id);

    // Find
    // Diagnostic: inspect raw documents in the collection to debug serialization issues
    {
    use mongodb::bson::{doc, Bson, Document};
    use futures::TryStreamExt;
        let coll = db.collection::<Document>("messages");
        let raw = coll
            .find_one(doc! { "_id": Bson::from(id.0) })
            .await
            .expect("raw find should succeed");
        println!("raw find for _id -> {:?}", raw);

        let mut cursor = coll.find(doc! {}).await.expect("list docs");
        while let Some(doc) = cursor.try_next().await.expect("cursor next") {
            println!("doc in collection: {:?}", doc);
        }
    }

    let found = repo.find_by_id(&id).await.expect("find should succeed");
    assert!(found.is_some(), "repo find_by_id returned None; inspect raw logs above");

    // List
    let (list, total) = repo.list(&GetPaginated::default()).await.expect("list should succeed");
    assert!(total >= 1);
    assert!(list.iter().any(|m| m.id == id));

    // Update
    let update_input = UpdateMessageInput { id, content: Some("updated mongo".into()), is_pinned: Some(true) };
    let updated = repo.update(update_input).await.expect("update should succeed");
    assert_eq!(updated.content, "updated mongo");

    // Delete
    repo.delete(&id).await.expect("delete should succeed");
    let after = repo.find_by_id(&id).await.expect("find after delete should succeed");
    assert!(after.is_none());

    // cleanup DB
    let _ = db.drop().await;

    // stop docker container if we started one
    if let Some(cid) = container_to_stop {
        let _ = stop_docker_container(&cid);
    }
}

fn stop_docker_container(container_id: &str) -> Result<(), String> {
    use std::process::Command;
    let out = Command::new("docker")
        .args(["rm", "-f", container_id])
        .output()
        .map_err(|e| format!("failed to stop docker container: {}", e))?;

    if !out.status.success() {
        return Err(format!("docker rm failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    Ok(())
}

async fn try_start_docker_mongo() -> Result<(String, String), String> {
    // Start a docker mongo container via the `docker` CLI and return (uri, container_id).
    use std::process::Command;

    // Ensure `docker` is available
    let docker_check = Command::new("docker").arg("version").output();
    if docker_check.is_err() {
        return Err("docker CLI not found".into());
    }

    // Start container with random host port mapping (-P) and capture container id
    let name = format!("test-mongo-{}", Uuid::new_v4().to_string());
    let run = Command::new("docker")
        .args(["run", "-d", "-P", "--rm", "--name", &name, "mongo:6.0"])
        .output()
        .map_err(|e| format!("failed to run docker: {}", e))?;

    if !run.status.success() {
        let stderr = String::from_utf8_lossy(&run.stderr);
        return Err(format!("docker run failed: {}", stderr));
    }

    let container_id = String::from_utf8_lossy(&run.stdout).trim().to_string();

    // Get mapped port for 27017
    let port_out = Command::new("docker")
        .args(["port", &container_id, "27017"])
        .output()
        .map_err(|e| format!("failed to query docker port: {}", e))?;

    if !port_out.status.success() {
        // Try a short wait and retry once
        std::thread::sleep(std::time::Duration::from_millis(200));
        let port_out = Command::new("docker")
            .args(["port", &container_id, "27017"])
            .output()
            .map_err(|e| format!("failed to query docker port: {}", e))?;
        if !port_out.status.success() {
            let stderr = String::from_utf8_lossy(&port_out.stderr);
            // cleanup container
            let _ = Command::new("docker").args(["rm", "-f", &container_id]).output();
            return Err(format!("docker port query failed: {}", stderr));
        }
        let out = String::from_utf8_lossy(&port_out.stdout);
        let host_port = out.trim().rsplit(':').next().ok_or_else(|| "failed to parse docker port output".to_string())?;
        let uri = format!("mongodb://127.0.0.1:{}", host_port);
        return Ok((uri, container_id));
    }

    let out = String::from_utf8_lossy(&port_out.stdout);
    let host_port = out.trim().rsplit(':').next().ok_or_else(|| "failed to parse docker port output".to_string())?;
    let uri = format!("mongodb://127.0.0.1:{}", host_port);
    Ok((uri, container_id))
}
