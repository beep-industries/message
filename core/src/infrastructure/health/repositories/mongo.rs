use std::future::Future;

use mongodb::{Database, bson::doc};

use crate::domain::health::{entities::IsHealthy, port::HealthRepository};

#[derive(Clone)]
pub struct MongoHealthRepository {
    db: Database,
}

impl MongoHealthRepository {
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }
}

impl HealthRepository for MongoHealthRepository {
    fn ping(&self) -> impl Future<Output = IsHealthy> + Send {
        let db = self.db.clone();

        async move {
            // MongoDB 3.x: run_command takes ONLY the command document
            let result = db.run_command(doc! { "ping": 1 }).await;
            IsHealthy::new(result.is_ok())
        }
    }
}
