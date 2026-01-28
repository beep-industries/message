pub trait AttachmentRepository: Send + Sync {
    fn get_signed_url(
        &self,
        id: String,
        verb: ContentVerb,
    ) -> impl Future<Output = Result<Attachment, CoreError>> + Send;

    fn put_attachment(&self) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;
    fn get_attachment(&self, id: String) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;
}
