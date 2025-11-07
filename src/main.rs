use std::sync::Arc;
use dotenv::dotenv;

use actix_web::{App, HttpServer};

use message::adapters::http;
use message::repositories::message::Repo as MongoRepo;
use message::usecases::MessageService;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize Mongo-backed repository from env MONGO_URL
    let repo_impl = match MongoRepo::from_env().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to initialize repository: {}", e);
            std::process::exit(1);
        }
    };

    let repo: Arc<dyn message::ports::MessageRepository> = Arc::new(repo_impl);

    let svc = MessageService::new(repo);

    println!("Starting server at http://127.0.0.1:3000");

    HttpServer::new(move || App::new().configure(|cfg| http::configure(cfg, svc.clone())))
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
