use communities_core::infrastructure::message::repositories::mongo::MongoMessageRepository;
use communities_core::domain::message::ports::MessageRepository;
use communities_core::domain::message::entities::{InsertMessageInput, Attachment, AttachmentId, ChannelId, AuthorId, MessageId, UpdateMessageInput};
use communities_core::domain::common::GetPaginated;
use mongodb::{Client, options::ClientOptions};
use uuid::Uuid;

/// Integration test for MongoMessageRepository.
/// Requires environment variable `MONGO_TEST_URI` to be set (e.g. mongodb://localhost:27017).
#[tokio::test]
async fn mongo_repository_crud_flow() {
    let uri = std::env::var("MONGO_TEST_URI").unwrap_or_default();
    if uri.is_empty() {
        eprintln!("Skipping Mongo integration test because MONGO_TEST_URI is not set");
        return;
    }

    let db_name = std::env::var("MONGO_TEST_DB").unwrap_or_else(|_| "message_test_db".into());

    let mut opts = ClientOptions::parse(&uri).await.expect("parse options");
    opts.app_name = Some("mongo_repo_integration_test".to_string());
    let client = Client::with_options(opts).expect("create client");
    let db = client.database(&db_name);

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
    let found = repo.find_by_id(&id).await.expect("find should succeed");
    assert!(found.is_some());

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

    // cleanup
    let _ = db.drop().await;
}
