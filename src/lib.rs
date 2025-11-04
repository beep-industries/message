pub mod domain;
pub mod ports;
pub mod usecases;
pub mod adapters {
    pub mod http;
}
pub mod repositories {
    pub mod message;
}

// Re-export commonly used items for ergonomics
pub use domain::*;
pub use ports::*;
pub use usecases::*;
pub use repositories::*;