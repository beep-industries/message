# RabbitMQ Integration for Message Service

## Overview

This implementation adds RabbitMQ message publishing to the message service using the Outbox pattern for reliable event delivery.

## Architecture

### Components

1. **Event Structures** (`core/src/domain/message/events.rs`)
   - `CreateMessageEvent`: Matches the protobuf schema from the real-time service
   - `NotifyEntry`: Represents notification targets
   - `AttachmentProto`: Attachment data for events

2. **RabbitMQ Publisher** (`core/src/infrastructure/rabbitmq/publisher.rs`)
   - Connection management with auto-reconnect
   - Exchange declaration (Topic type)
   - Message publishing with persistence
   - Thread-safe using Arc<RwLock>

3. **Outbox Relay Service** (`core/src/infrastructure/rabbitmq/relay.rs`)
   - Polls MongoDB outbox collection for READY messages
   - Publishes to RabbitMQ
   - Updates status to SENT or FAILED
   - Runs as background task

4. **Integration**
   - Message repository writes events to outbox on creation
   - Relay service reads from outbox and publishes to RabbitMQ
   - Transactional safety via outbox pattern

## Configuration

### Environment Variables

```bash
# .env file
RABBITMQ_URL=amqp://guest:guest@localhost:5672
```

### Routing Configuration

```yaml
# config/routing.yaml
create_message:
  exchange: "notifications" # Topic exchange for notifications
   routing_key: "message.create" # Routing key
```

## Flow

1. **Message Creation**

   ```
   HTTP POST /messages
   ↓
   MessageService.create_message()
   ↓
   MessageRepository.insert()
   ↓
   Write to MongoDB messages collection
   ↓
   Write CreateMessageEvent to outbox (READY status)
   ```

2. **Event Publishing** (Background)
   ```
   OutboxRelayService (polling every 1s)
   ↓
   Query outbox for READY messages
   ↓
   Publish to RabbitMQ (exchange: "notifications", key: "message.create")
   ↓
   Update status to SENT (or FAILED on error)
   ```

## RabbitMQ Setup

### Exchange Configuration

- **Name**: `notifications`
- **Type**: `topic`
- **Durable**: `true`

### Message Format

Messages are published as JSON with content-type `application/json`:

```json
{
  "message_id": "uuid-string",
  "channel_id": "uuid-string",
  "author_id": "uuid-string",
  "content": "message text",
  "reply_to_message_id": "uuid-string or null",
  "attachments": [
    {
      "id": "uuid-string",
      "name": "filename",
      "url": "file-url"
    }
  ],
  "notify_entries": []
}
```

## Consumer Setup (Example for notifications service)

```
Queue: notifications.messages.created
Exchange: notifications (Topic)
Binding Key: message.create
```

## Benefits of Outbox Pattern

1. **Transactional Safety**: Message and event are written in same DB transaction
2. **At-least-once delivery**: Events won't be lost even if RabbitMQ is down
3. **Retry Logic**: Failed messages can be retried
4. **Audit Trail**: All events are logged in outbox collection

## Monitoring

- Check outbox collection for FAILED status messages
- Monitor relay service logs for publishing errors
- RabbitMQ management UI for message flow

## Dependencies Added

- `lapin = "3.7.2"` - RabbitMQ client
- `prost = "0.14.3"` - Protobuf support
- `tokio = { version = "1", features = ["sync", "time"] }` - Async runtime

## Testing

Integration tests updated to pass routing configuration:

- `core/tests/mongo_repo_integration.rs`
- `api/tests/http_messages_integration.rs`
