use communities_core::domain::message::entities::{InsertMessageInput, MessageId, ChannelId, AuthorId, Attachment, AttachmentId, UpdateMessageInput};
use communities_core::domain::message::ports::{MockMessageRepository, MessageService};
use communities_core::domain::health::port::MockHealthRepository;
use communities_core::domain::common::CoreError;
use communities_core::domain::common::services::Service;
use uuid::Uuid;

#[tokio::test]
async fn service_create_get_update_delete_flow() {
    let repo = MockMessageRepository::new();
    let health = MockHealthRepository::new();

    let service = Service::new(repo.clone(), health);

    let id = MessageId::from(Uuid::new_v4());
    let channel = ChannelId::from(Uuid::new_v4());
    let author = AuthorId::from(Uuid::new_v4());

    let input = InsertMessageInput {
        id,
        channel_id: channel,
        author_id: author,
        content: "service message".into(),
        reply_to_message_id: None,
        attachments: vec![Attachment { id: AttachmentId::from(Uuid::new_v4()), name: "a".into(), url: "u".into() }],
    };

    // create
    let created = service.create_message(input.clone()).await.expect("create should work");
    assert_eq!(created.id, id);

    // get
    let got = service.get_message(&id).await.expect("get should work");
    assert_eq!(got.content, "service message");

    // update
    let update = UpdateMessageInput { id, content: Some("changed".into()), is_pinned: Some(false) };
    let updated = service.update_message(update).await.expect("update should work");
    assert_eq!(updated.content, "changed");

    // delete
    service.delete_message(&id).await.expect("delete should work");

    // get after delete -> not found
    let res = service.get_message(&id).await;
    assert!(matches!(res, Err(CoreError::MessageNotFound { .. })));
}

#[tokio::test]
async fn create_invalid_message_name_rejected() {
    let repo = MockMessageRepository::new();
    let health = MockHealthRepository::new();
    let service = Service::new(repo, health);

    let input = InsertMessageInput {
        id: MessageId::from(Uuid::new_v4()),
        channel_id: ChannelId::from(Uuid::new_v4()),
        author_id: AuthorId::from(Uuid::new_v4()),
        content: "  ".into(),
        reply_to_message_id: None,
        attachments: vec![],
    };

    let res = service.create_message(input).await;
    assert!(matches!(res, Err(CoreError::InvalidMessageName)));
}
