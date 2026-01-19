use clap::Parser;
use clap::ValueEnum;
use communities_core::application::MessageRoutingInfos;
use std::path::PathBuf;

#[derive(Clone, Parser, Debug, Default)]
#[command(name = "communities-api")]
#[command(about = "Communities API Message", long_about = None)]
pub struct Config {
    #[command(flatten)]
    pub database: DatabaseConfig,

    #[command(flatten)]
    pub jwt: JwtConfig,

    #[command(flatten)]
    pub keycloak: KeycloakConfig,

    #[command(flatten)]
    pub message: MessageConfig,

    #[arg(
        long = "routing-config",
        env = "ROUTING_CONFIG_PATH",
        default_value = "config/routing.yaml"
    )]
    pub routing_config_path: PathBuf,

    #[arg(skip)]
    pub routing: MessageRoutingInfos,

    #[arg(
        long = "environment",
        env = "ENVIRONMENT",
        default_value = "development"
    )]
    pub environment: Environment,
}

impl Config {
    /// Load routing configuration from YAML file
    pub fn load_routing(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let yaml_content = std::fs::read_to_string(&self.routing_config_path)?;
        self.routing = serde_yaml::from_str(&yaml_content)?;
        Ok(())
    }
}

#[derive(Clone, Parser, Debug, Default)]
pub struct KeycloakConfig {
    #[arg(
        long = "keycloak-internal-url",
        env = "KEYCLOAK_INTERNAL_URL",
        default_value = "localhost"
    )]
    pub internal_url: String,

    #[arg(
        long = "keycloak-realm",
        env = "KEYCLOAK_REALM",
        default_value = "user"
    )]
    pub realm: String,
}
#[derive(Clone, Parser, Debug, Default)]
pub struct DatabaseConfig {
    #[arg(
        long = "database-uri",
        env = "DATABASE_URI",
        default_value = "mongodb://localhost:27017/messages"
    )]
    pub mongo_uri: String,

    #[arg(
        long = "database-name",
        env = "DATABASE_NAME",
        default_value = "messages",
        value_name = "database_name"
    )]
    pub mongo_db_name: String,
}

#[derive(Clone, Parser, Debug, Default)]
pub struct JwtConfig {
    #[arg(
        long = "jwt-secret-key",
        env = "JWT_SECRET_KEY",
        name = "jwt_secret_key"
    )]
    pub secret_key: String,
}

#[derive(Clone, Parser, Debug, Default)]
pub struct MessageConfig {
    #[arg(
        long = "message-api-port",
        env = "API_PORT",
        default_value = "8080",
        name = "api_port"
    )]
    pub api_port: u16,

    #[arg(
        long = "message-health-port",
        env = "HEALTH_PORT",
        default_value = "8081"
    )]
    pub health_port: u16,
}

#[derive(Clone, Debug, ValueEnum, Default)]
pub enum Environment {
    #[default]
    Development,
    Production,
    Test,
}
