use crate::domain::{
    CoreError,
    entities::{ChannelId, Message, MessageId},
};

pub trait MessageRepository: Send + Sync {
    fn get(
        &self,
        channel_id: &ChannelId,
        id: &MessageId,
    ) -> impl Future<Output = Result<Message, CoreError>> + Send;

    fn list(
        &self,
        channel: &ChannelId,
        limit: Option<u32>,
        before: Option<MessageId>,
    ) -> impl Future<Output = Result<(Vec<Message>, Option<MessageId>), CoreError>> + Send;

    fn delete(&self, message_id: &MessageId) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn pin_message(
        &self,
        message_id: &MessageId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_pinned_messages(
        &self,
        channel_id: &ChannelId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> impl Future<Output = Result<(Vec<Message>, usize), CoreError>> + Send;
}
