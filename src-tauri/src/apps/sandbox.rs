use super::AppPermission;
use std::collections::{HashMap, HashSet};

pub struct PermissionSandbox {
    app_permissions: HashMap<String, HashSet<AppPermission>>,
}

impl PermissionSandbox {
    pub fn new() -> Self {
        Self {
            app_permissions: HashMap::new(),
        }
    }

    /// Load app permissions
    pub fn load_permissions(&mut self, app_id: &str, permissions: Vec<AppPermission>) {
        self.app_permissions
            .insert(app_id.to_string(), permissions.into_iter().collect());
    }

    /// Check if app has permission
    pub fn has_permission(&self, app_id: &str, permission: &AppPermission) -> bool {
        self.app_permissions
            .get(app_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }

    /// Enforce permission check (returns error if denied)
    pub fn require_permission(
        &self,
        app_id: &str,
        permission: &AppPermission,
    ) -> Result<(), PermissionError> {
        if self.has_permission(app_id, permission) {
            Ok(())
        } else {
            Err(PermissionError::Denied(format!(
                "App {} does not have permission: {:?}",
                app_id, permission
            )))
        }
    }

    /// Clear permissions for an app
    pub fn clear_permissions(&mut self, app_id: &str) {
        self.app_permissions.remove(app_id);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Permission denied: {0}")]
    Denied(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_sandbox() {
        let mut sandbox = PermissionSandbox::new();
        sandbox.load_permissions("app1", vec![AppPermission::Storage, AppPermission::GameSession]);

        assert!(sandbox.has_permission("app1", &AppPermission::Storage));
        assert!(sandbox.has_permission("app1", &AppPermission::GameSession));
        assert!(!sandbox.has_permission("app1", &AppPermission::Notify));
        assert!(!sandbox.has_permission("app2", &AppPermission::Storage));
    }

    #[test]
    fn test_require_permission_denied() {
        let sandbox = PermissionSandbox::new();
        assert!(sandbox.require_permission("app1", &AppPermission::Storage).is_err());
    }

    #[test]
    fn test_clear_permissions() {
        let mut sandbox = PermissionSandbox::new();
        sandbox.load_permissions("app1", vec![AppPermission::Storage]);
        assert!(sandbox.has_permission("app1", &AppPermission::Storage));

        sandbox.clear_permissions("app1");
        assert!(!sandbox.has_permission("app1", &AppPermission::Storage));
    }
}
