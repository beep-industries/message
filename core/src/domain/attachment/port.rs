use crate::{
    domain::{
        common::CoreError, message::entities::{Attachment, AttachmentId},
    },
    infrastructure::attachments::repositories::entities::ContentVerb,
};

#[async_trait::async_trait]
pub trait AttachmentService: Send + Sync {
    async fn create_attachment(&self) -> Result<Attachment, CoreError>;
    async fn get_attachment(&self, id: String) -> Result<Attachment, CoreError>;
}

pub trait AttachmentRepository: Send + Sync {
    fn get_signed_url(
        &self,
        id: String,
        verb: ContentVerb,
    ) -> impl Future<Output = Result<Attachment, CoreError>> + Send;

    fn post_attachment(&self) -> impl Future<Output = Result<Attachment, CoreError>> + Send;
    fn get_attachment(
        &self,
        id: String,
    ) -> impl Future<Output = Result<Attachment, CoreError>> + Send;
}

pub struct MockAttachmentRepository;

impl MockAttachmentRepository {
    pub fn new() -> Self {
        Self
    }
}

impl AttachmentRepository for MockAttachmentRepository {
    fn get_signed_url(
        &self,
        id: String,
        _verb: ContentVerb,
    ) -> impl Future<Output = Result<Attachment, CoreError>> + Send {
        let attachment = Attachment {
            id: AttachmentId::try_from(id).expect("Invalid UUID string"),
            url: "http://example.com/signed_url".to_string(),
        };
        async move { Ok(attachment) }
    }

    fn post_attachment(&self) -> impl Future<Output = Result<Attachment, CoreError>> + Send {
        let id = uuid::Uuid::new_v4().to_string();
        let attachment = Attachment {
            id: AttachmentId::try_from(id).expect("Invalid UUID string"),
            url: "http://example.com/put_signed_url".to_string(),
        };
        async move { Ok(attachment) }
    }

    fn get_attachment(
        &self,
        id: String,
    ) -> impl Future<Output = Result<Attachment, CoreError>> + Send {
        let attachment = Attachment {
            id: AttachmentId::try_from(id.clone()).expect("Invalid UUID string"),
            url: format!("http://example.com/attachment/{}", AttachmentId::try_from(id).expect("Invalid UUID string")),
        };
        async move { Ok(attachment) }
    }
}