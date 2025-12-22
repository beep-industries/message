pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types for convenience
pub use application::{CommunitiesService, create_repositories};
pub use domain::common::services::Service;
pub use infrastructure::health::repositories::mongo::MongoHealthRepository;
pub use infrastructure::message::repositories::mongo::MongoMessageRepository;

// Re-export outbox pattern primitives
pub use infrastructure::outbox::write_outbox_event;
