use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::domain::common::CoreError;

/// RabbitMQ publisher for publishing domain events
#[derive(Clone)]
pub struct RabbitMqPublisher {
    connection: Arc<RwLock<Option<Connection>>>,
    channel: Arc<RwLock<Option<Channel>>>,
    url: String,
}

impl RabbitMqPublisher {
    /// Create a new RabbitMQ publisher
    pub fn new(url: String) -> Self {
        Self {
            connection: Arc::new(RwLock::new(None)),
            channel: Arc::new(RwLock::new(None)),
            url,
        }
    }

    /// Connect to RabbitMQ and create a channel
    pub async fn connect(&self) -> Result<(), CoreError> {
        info!("Connecting to RabbitMQ at {}", self.url);

        let conn = Connection::connect(&self.url, ConnectionProperties::default())
            .await
            .map_err(|e| CoreError::RabbitMqError {
                msg: format!("Failed to connect to RabbitMQ: {}", e),
            })?;

        let channel = conn
            .create_channel()
            .await
            .map_err(|e| CoreError::RabbitMqError {
                msg: format!("Failed to create channel: {}", e),
            })?;

        *self.connection.write().await = Some(conn);
        *self.channel.write().await = Some(channel);

        info!("Successfully connected to RabbitMQ");
        Ok(())
    }

    /// Ensure exchange exists (declare it if not)
    pub async fn declare_exchange(&self, exchange_name: &str) -> Result<(), CoreError> {
        let channel_guard = self.channel.read().await;
        let channel = channel_guard
            .as_ref()
            .ok_or_else(|| CoreError::RabbitMqError {
                msg: "Channel not initialized. Call connect() first.".to_string(),
            })?;

        channel
            .exchange_declare(
                exchange_name,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| CoreError::RabbitMqError {
                msg: format!("Failed to declare exchange {}: {}", exchange_name, e),
            })?;

        info!("Declared exchange: {}", exchange_name);
        Ok(())
    }

    /// Publish a message to an exchange with a routing key
    pub async fn publish(
        &self,
        exchange_name: &str,
        routing_key: &str,
        payload: Vec<u8>,
    ) -> Result<(), CoreError> {
        let channel_guard = self.channel.read().await;
        let channel = channel_guard
            .as_ref()
            .ok_or_else(|| CoreError::RabbitMqError {
                msg: "Channel not initialized. Call connect() first.".to_string(),
            })?;

        let properties = BasicProperties::default()
            .with_content_type("application/json".into())
            .with_delivery_mode(2); // persistent

        channel
            .basic_publish(
                exchange_name,
                routing_key,
                BasicPublishOptions::default(),
                &payload,
                properties,
            )
            .await
            .map_err(|e| CoreError::RabbitMqError {
                msg: format!(
                    "Failed to publish message to {}/{}: {}",
                    exchange_name, routing_key, e
                ),
            })?
            .await
            .map_err(|e| CoreError::RabbitMqError {
                msg: format!(
                    "Failed to confirm publish to {}/{}: {}",
                    exchange_name, routing_key, e
                ),
            })?;

        info!(
            "Published message to exchange: {}, routing_key: {}",
            exchange_name, routing_key
        );
        Ok(())
    }

    /// Check if the connection is alive
    pub async fn is_connected(&self) -> bool {
        let conn_guard = self.connection.read().await;
        if let Some(conn) = conn_guard.as_ref() {
            conn.status().connected()
        } else {
            false
        }
    }

    /// Reconnect if connection is lost
    pub async fn ensure_connected(&self) -> Result<(), CoreError> {
        if !self.is_connected().await {
            warn!("RabbitMQ connection lost. Reconnecting...");
            self.connect().await?;
        }
        Ok(())
    }

    /// Gracefully close the connection
    pub async fn close(&self) -> Result<(), CoreError> {
        let mut conn_guard = self.connection.write().await;
        if let Some(conn) = conn_guard.take() {
            conn.close(0, "Normal shutdown")
                .await
                .map_err(|e| CoreError::RabbitMqError {
                    msg: format!("Failed to close connection: {}", e),
                })?;
            info!("RabbitMQ connection closed");
        }
        Ok(())
    }
}
