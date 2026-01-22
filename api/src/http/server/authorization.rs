use std::sync::Arc;
use uuid::Uuid;

/// A small, local abstraction for authorization checks used by HTTP handlers.
///
/// We provide a DummyAuthz (allow-all) implementation by default, and a
/// SpiceDB-backed implementation when the `spicedb` feature is enabled.
pub enum Resource {
    Channel(Uuid),
    User(Uuid),
}

#[derive(Clone, Copy, Debug)]
pub enum Permission {
    ViewChannels,
    SendMessages,
    ManageMessages,
    ManageChannels,
}

/// Simple error type for authz failures.
#[derive(Debug)]
pub struct AuthzError(pub String);

#[async_trait::async_trait]
pub trait Authorization: Send + Sync + 'static {
    async fn check(&self, actor: Uuid, permission: Permission, resource: Resource) -> Result<bool, AuthzError>;
}

#[derive(Clone)]
pub struct DummyAuthz;

impl DummyAuthz {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Authorization for DummyAuthz {
    async fn check(&self, _actor: Uuid, _permission: Permission, _resource: Resource) -> Result<bool, AuthzError> {
        // permissive default for local dev/tests
        Ok(true)
    }
}

/// Public wrapper so AppState can hold a shared authorization client.
pub type DynAuthz = Arc<dyn Authorization>;

mod spicedb_impl {
    use super::*;
        use beep_authz::{Permissions as ExtPermissions, SpiceDbConfig as ExtConfig, SpiceDbObject, SpiceDbRepository};

    #[derive(Clone)]
    pub struct SpiceDbAuthz {
        repo: SpiceDbRepository,
    }

    impl SpiceDbAuthz {
        pub async fn new(cfg: ExtConfig) -> Result<Self, AuthzError> {
            let repo = SpiceDbRepository::new(cfg).await.map_err(|e| AuthzError(format!("spicedb init error: {}", e)))?;
            Ok(Self { repo })
        }
    }

    fn map_permission(p: Permission) -> ExtPermissions {
        match p {
            Permission::ViewChannels => ExtPermissions::ViewChannels,
            Permission::SendMessages => ExtPermissions::SendMessages,
            Permission::ManageMessages => ExtPermissions::ManageMessages,
            Permission::ManageChannels => ExtPermissions::ManageChannels,
        }
    }

    #[async_trait::async_trait]
    impl Authorization for SpiceDbAuthz {
        async fn check(&self, actor: Uuid, permission: Permission, resource: Resource) -> Result<bool, AuthzError> {
            let ext_perm = map_permission(permission);
            let actor_obj = SpiceDbObject::User(actor.to_string());

            let resource_obj = match resource {
                Resource::Channel(id) => SpiceDbObject::Channel(id.to_string()),
                Resource::User(id) => SpiceDbObject::User(id.to_string()),
            };

            let res = self.repo.check_permissions(resource_obj, ext_perm, actor_obj).await;
            Ok(res.has_permissions())
        }
    }

    // re-export for use by the app
    pub use SpiceDbAuthz as SpiceDbAuthzImpl;
}

// Re-export the SpiceDbConfig from the external crate directly (public)
pub use beep_authz::config::SpiceDbConfig;
pub use spicedb_impl::SpiceDbAuthzImpl as SpiceDbAuthz;

