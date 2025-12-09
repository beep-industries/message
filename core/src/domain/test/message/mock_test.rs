use crate::{
    Service,
    domain::{
        common::GetPaginated,
        health::port::MockHealthRepository,
        message::{
            entities::{
                InsertMessageInput, MessageId, MessageVisibility, OwnerId, UpdateMessageInput,
            },
            ports::{MessageRepository, MessageService, MockMessageRepository},
        },
        message_member::ports::MockMemberRepository,
    },
};
use uuid::Uuid;

// == Create Message Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_create_message_success() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let input = InsertMessageInput {
        name: "Test Message".to_string(),
        owner_id: OwnerId::from(Uuid::new_v4()),
        picture_url: Some("https://example.com/picture.png".to_string()),
        banner_url: Some("https://example.com/banner.png".to_string()),
        description: Some("A test message".to_string()),
        visibility: MessageVisibility::Public,
    };

    let message = service
        .create_message(input.clone())
        .await
        .expect("create_message returned an error");

    assert_eq!(
        message.name, "Test Message",
        "Expected correct message name"
    );
    assert_eq!(
        message.owner_id, input.owner_id,
        "Expected correct owner ID"
    );
    assert_eq!(
        message.visibility,
        MessageVisibility::Public,
        "Expected public visibility"
    );
    assert_eq!(
        message.picture_url,
        Some("https://example.com/picture.png".to_string()),
        "Expected correct picture URL"
    );
    assert_eq!(
        message.banner_url,
        Some("https://example.com/banner.png".to_string()),
        "Expected correct banner URL"
    );
    assert_eq!(
        message.description,
        Some("A test message".to_string()),
        "Expected correct description"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_create_message_fail_empty_name() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let input = InsertMessageInput {
        name: "".to_string(),
        owner_id: OwnerId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: MessageVisibility::Public,
    };

    let error = service
        .create_message(input)
        .await
        .expect_err("create_message should have returned an error");

    assert_eq!(
        error.to_string(),
        "Message name cannot be empty",
        "Expected invalid message name error"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_create_message_fail_whitespace_name() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let input = InsertMessageInput {
        name: "   ".to_string(),
        owner_id: OwnerId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: MessageVisibility::Public,
    };

    let error = service
        .create_message(input)
        .await
        .expect_err("create_message should have returned an error");

    assert_eq!(
        error.to_string(),
        "Message name cannot be empty",
        "Expected invalid message name error"
    );

    Ok(())
}

// == Get Message Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_get_message_success() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo.clone(),
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a message using repository
    let input = InsertMessageInput {
        name: "Test Message".to_string(),
        owner_id: OwnerId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: MessageVisibility::Public,
    };
    let created_message = message_mock_repo.insert(input).await?;

    // Get the message
    let message = service
        .get_message(&created_message.id)
        .await
        .expect("get_message returned an error");

    assert_eq!(message.id, created_message.id, "Expected same message ID");
    assert_eq!(
        message.name, "Test Message",
        "Expected correct message name"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_get_message_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let non_existent_id = MessageId::from(Uuid::new_v4());
    let error = service
        .get_message(&non_existent_id)
        .await
        .expect_err("get_message should have returned an error");

    assert!(
        error.to_string().contains("not found"),
        "Expected message not found error"
    );

    Ok(())
}

// == List Messages Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_list_messages_success() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo.clone(),
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert multiple messages
    for i in 1..=3 {
        let input = InsertMessageInput {
            name: format!("Test Message {}", i),
            owner_id: OwnerId::from(Uuid::new_v4()),
            picture_url: None,
            banner_url: None,
            description: None,
            visibility: MessageVisibility::Public,
        };
        message_mock_repo.insert(input).await?;
    }

    let (messages, total) = service
        .list_messages(&GetPaginated::default())
        .await
        .expect("list_messages returned an error");

    assert_eq!(messages.len(), 3, "Expected 3 messages in the list");
    assert_eq!(total, 3, "Expected total count to be 3");

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_list_messages_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo.clone(),
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert 25 messages
    for i in 1..=25 {
        let input = InsertMessageInput {
            name: format!("Test Message {}", i),
            owner_id: OwnerId::from(Uuid::new_v4()),
            picture_url: None,
            banner_url: None,
            description: None,
            visibility: MessageVisibility::Public,
        };
        message_mock_repo.insert(input).await?;
    }

    // Test page 1
    let pagination1 = GetPaginated { page: 1, limit: 10 };
    let (messages1, total1) = service
        .list_messages(&pagination1)
        .await
        .expect("list_messages page 1 returned an error");

    assert_eq!(messages1.len(), 10, "Expected 10 messages on page 1");
    assert_eq!(total1, 25, "Expected total count to be 25");

    // Test page 2
    let pagination2 = GetPaginated { page: 2, limit: 10 };
    let (messages2, total2) = service
        .list_messages(&pagination2)
        .await
        .expect("list_messages page 2 returned an error");

    assert_eq!(messages2.len(), 10, "Expected 10 messages on page 2");
    assert_eq!(total2, 25, "Expected total count to be 25");

    // Test page 3
    let pagination3 = GetPaginated { page: 3, limit: 10 };
    let (messages3, total3) = service
        .list_messages(&pagination3)
        .await
        .expect("list_messages page 3 returned an error");

    assert_eq!(messages3.len(), 5, "Expected 5 messages on page 3");
    assert_eq!(total3, 25, "Expected total count to be 25");

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_list_messages_empty() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let (messages, total) = service
        .list_messages(&GetPaginated::default())
        .await
        .expect("list_messages returned an error");

    assert_eq!(messages.len(), 0, "Expected empty message list");
    assert_eq!(total, 0, "Expected total count to be 0");

    Ok(())
}

// == Update Message Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_update_message_success() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo.clone(),
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a message
    let input = InsertMessageInput {
        name: "Original Message".to_string(),
        owner_id: OwnerId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: MessageVisibility::Public,
    };
    let created_message = message_mock_repo.insert(input).await?;

    // Update the message
    let update_input = UpdateMessageInput {
        id: created_message.id.clone(),
        name: Some("Updated Message".to_string()),
        picture_url: None,
        banner_url: None,
        description: Some("Updated description".to_string()),
        visibility: Some(MessageVisibility::Private),
    };

    let updated_message = service
        .update_message(update_input)
        .await
        .expect("update_message returned an error");

    assert_eq!(
        updated_message.name, "Updated Message",
        "Expected updated name"
    );
    assert_eq!(
        updated_message.description,
        Some("Updated description".to_string()),
        "Expected updated description"
    );
    assert_eq!(
        updated_message.visibility,
        MessageVisibility::Private,
        "Expected updated visibility"
    );
    assert!(
        updated_message.updated_at.is_some(),
        "Expected updated_at to be set"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_message_partial_update() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo.clone(),
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a message
    let input = InsertMessageInput {
        name: "Original Message".to_string(),
        owner_id: OwnerId::from(Uuid::new_v4()),
        picture_url: Some("https://example.com/original.png".to_string()),
        banner_url: None,
        description: Some("Original description".to_string()),
        visibility: MessageVisibility::Public,
    };
    let created_message = message_mock_repo.insert(input).await?;

    // Update only the name
    let update_input = UpdateMessageInput {
        id: created_message.id.clone(),
        name: Some("Updated Name Only".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };

    let updated_message = service
        .update_message(update_input)
        .await
        .expect("update_message returned an error");

    assert_eq!(
        updated_message.name, "Updated Name Only",
        "Expected updated name"
    );
    assert_eq!(
        updated_message.description,
        Some("Original description".to_string()),
        "Expected unchanged description"
    );
    assert_eq!(
        updated_message.picture_url,
        Some("https://example.com/original.png".to_string()),
        "Expected unchanged picture URL"
    );
    assert_eq!(
        updated_message.visibility,
        MessageVisibility::Public,
        "Expected unchanged visibility"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_message_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let update_input = UpdateMessageInput {
        id: MessageId::from(Uuid::new_v4()),
        name: Some("Updated Message".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };

    let error = service
        .update_message(update_input)
        .await
        .expect_err("update_message should have returned an error");

    assert!(
        error.to_string().contains("not found"),
        "Expected message not found error"
    );

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_update_message_fail_empty_name() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo.clone(),
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a message
    let input = InsertMessageInput {
        name: "Original Message".to_string(),
        owner_id: OwnerId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: MessageVisibility::Public,
    };
    let created_message = message_mock_repo.insert(input).await?;

    // Try to update with empty name
    let update_input = UpdateMessageInput {
        id: created_message.id.clone(),
        name: Some("".to_string()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: None,
    };

    let error = service
        .update_message(update_input)
        .await
        .expect_err("update_message should have returned an error");

    assert_eq!(
        error.to_string(),
        "Message name cannot be empty",
        "Expected invalid message name error"
    );

    Ok(())
}

// == Delete Message Tests ==

#[tokio::test]
#[cfg(test)]
async fn test_delete_message_success() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo.clone(),
        health_mock_repo,
        MockMemberRepository::new(),
    );

    // Insert a message
    let input = InsertMessageInput {
        name: "Test Message".to_string(),
        owner_id: OwnerId::from(Uuid::new_v4()),
        picture_url: None,
        banner_url: None,
        description: None,
        visibility: MessageVisibility::Public,
    };
    let created_message = message_mock_repo.insert(input).await?;

    // Delete the message
    service
        .delete_message(&created_message.id)
        .await
        .expect("delete_message returned an error");

    // Verify message is deleted
    let deleted_message = message_mock_repo.find_by_id(&created_message.id).await?;
    assert!(deleted_message.is_none(), "Expected message to be deleted");

    Ok(())
}

#[tokio::test]
#[cfg(test)]
async fn test_delete_message_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let message_mock_repo = MockMessageRepository::new();
    let health_mock_repo = MockHealthRepository::new();
    let service = Service::new(
        message_mock_repo,
        health_mock_repo,
        MockMemberRepository::new(),
    );

    let non_existent_id = MessageId::from(Uuid::new_v4());
    let error = service
        .delete_message(&non_existent_id)
        .await
        .expect_err("delete_message should have returned an error");

    assert!(
        error.to_string().contains("not found"),
        "Expected message not found error"
    );

    Ok(())
}
