use communities_core::domain::message::ports::{MockMessageRepository, MessageRepository};
use communities_core::domain::message::entities::{InsertMessageInput, Attachment, AttachmentId, ChannelId, AuthorId, MessageId, UpdateMessageInput};
use communities_core::domain::common::{GetPaginated, CoreError};
use uuid::Uuid;

#[tokio::test]
async fn mock_repo_crud_flow() {
    let repo = MockMessageRepository::new();

    let id = MessageId::from(Uuid::new_v4());
    let channel = ChannelId::from(Uuid::new_v4());
    let author = AuthorId::from(Uuid::new_v4());

    let input = InsertMessageInput {
        id,
        channel_id: channel,
        author_id: author,
        content: "hello world".to_string(),
        reply_to_message_id: None,
        attachments: vec![Attachment { id: AttachmentId::from(Uuid::new_v4()), name: "file.txt".into(), url: "http://example.com/file.txt".into() }],
    };

    // Insert
    let inserted = repo.insert(input.clone()).await.expect("insert should succeed");
    assert_eq!(inserted.id, id);
    assert_eq!(inserted.content, "hello world");

    // Find
    let found = repo.find_by_id(&id).await.expect("find should succeed");
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, id);

    // List
    let (list, total) = repo.list(&GetPaginated::default()).await.expect("list should succeed");
    assert!(total >= 1);
    assert!(list.iter().any(|m| m.id == id));

    // Update
    let update_input = UpdateMessageInput { id, content: Some("updated".into()), is_pinned: Some(true) };
    let updated = repo.update(update_input).await.expect("update should succeed");
    assert_eq!(updated.content, "updated");
    assert!(updated.is_pinned);

    // Delete
    repo.delete(&id).await.expect("delete should succeed");
    let after = repo.find_by_id(&id).await.expect("find after delete should succeed");
    assert!(after.is_none());

    // Delete non-existent -> MessageNotFound
    let missing_id = MessageId::from(Uuid::new_v4());
    let res = repo.delete(&missing_id).await;
    assert!(matches!(res, Err(CoreError::MessageNotFound { .. })));
}
