use reqwest::{Client, Url};
use tracing::{debug, error};

use crate::{
    domain::{
        attachment::{entities::PresignedUrl, port::AttachmentRepository}, common::CoreError, message::entities::Attachment,
    },
    infrastructure::attachments::repositories::entities::{ContentVerb, RequestSignUrl},
};

#[derive(Debug, Clone)]
pub struct ReqwestAttachmentRepository {
    content_url: String,
    client: Client,
}

impl ReqwestAttachmentRepository {
    pub fn new(content_url: &String) -> Self {
        Self {
            content_url: content_url.clone(),
            client: Client::new()
        }
    }
}

impl AttachmentRepository for ReqwestAttachmentRepository {
    async fn get_signed_url(&self, id: String, verb: ContentVerb) -> Result<Attachment, CoreError> {
        let content_url =
            Url::parse(&self.content_url).map_err(|_| CoreError::ParseContentUrl {
                part: self.content_url.clone(),
            })?;

        let uuid = if ContentVerb::Put == verb {
            uuid::Uuid::new_v4().to_string()
        } else {
            id
        };  

        let formatted_prefix = format!("attachment/{}", uuid);
        let url = content_url.join(formatted_prefix.as_str()).map_err(|_| {
            CoreError::ParseContentUrl {
                part: uuid.to_string(),
            }
        })?;

        let presigned_url = self
            .client
            .post(url)
            .json(&RequestSignUrl::from(verb))
            .send()
            .await
            .map_err(|e| {
                error!("{}", e);
                return CoreError::FailedToGetSignedUrl { err: e.to_string() };
            })?
            .json::<PresignedUrl>()
            .await
            .map_err(|e| {
                debug!("{}", e);
                return CoreError::FailedToGetSignedUrl { err: e.to_string() };
            })?;

        let uuid = uuid::Uuid::parse_str(&uuid).map_err(|_| CoreError::ParseContentUrl {
            part: uuid.to_string(),
        })?;

        Ok(Attachment { id: uuid.into(), url: presigned_url.url })
    }

    async fn post_attachment(&self) -> Result<Attachment, CoreError> {
        self.get_signed_url("".to_string(), ContentVerb::Put)
            .await
    }

    async fn get_attachment(&self, id: String) -> Result<Attachment, CoreError> {
        let attachment = self
            .get_signed_url(id, ContentVerb::Get)
            .await?;
        Ok(attachment)
    }
}
