use serde_json::{json, Value};
use uuid::Uuid;

/// Builder for constructing audit log details with diff information.
///
/// Usage:
/// ```
/// use cradle_backend::services::audit_change_builder::AuditChangeBuilder;
/// use serde_json::json;
/// use uuid::Uuid;
///
/// let user_id = Uuid::new_v4();
/// let old_json = json!({"name": "Alice"});
/// let new_json = json!({"name": "Bob"});
/// let details = AuditChangeBuilder::new("user", user_id)
///     .old(old_json)
///     .new_value(new_json)
///     .build();
/// ```
pub struct AuditChangeBuilder {
    resource_type: String,
    resource_id: Uuid,
    old_value: Option<Value>,
    new_value: Option<Value>,
}

impl AuditChangeBuilder {
    /// Create a new builder for the given resource type and ID
    pub fn new(resource_type: &str, resource_id: Uuid) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id,
            old_value: None,
            new_value: None,
        }
    }

    /// Set the old (previous) value
    pub fn old(mut self, value: Value) -> Self {
        self.old_value = Some(value);
        self
    }

    /// Set the new (updated) value
    pub fn new_value(mut self, value: Value) -> Self {
        self.new_value = Some(value);
        self
    }

    /// Build the audit details JSON with old_value, new_value, and a changes diff map.
    ///
    /// The `changes` map contains `{ field: [old, new] }` for each field
    /// that differs between old and new values.
    pub fn build(self) -> Value {
        let mut changes = serde_json::Map::new();

        if let (Some(old), Some(new)) = (&self.old_value, &self.new_value) {
            // If both are objects, compute per-field diff
            if let (Value::Object(old_map), Value::Object(new_map)) = (old, new) {
                // Collect all keys from both maps
                let mut all_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
                for k in old_map.keys() {
                    all_keys.insert(k.clone());
                }
                for k in new_map.keys() {
                    all_keys.insert(k.clone());
                }

                for key in all_keys {
                    let old_val = old_map.get(&key).unwrap_or(&Value::Null);
                    let new_val = new_map.get(&key).unwrap_or(&Value::Null);

                    if old_val != new_val {
                        changes.insert(
                            key,
                            json!([old_val, new_val]),
                        );
                    }
                }
            }
        }

        json!({
            "resource_type": self.resource_type,
            "resource_id": self.resource_id.to_string(),
            "old_value": self.old_value,
            "new_value": self.new_value,
            "changes": changes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_diff() {
        let user_id = Uuid::new_v4();
        let details = AuditChangeBuilder::new("user", user_id)
            .old(json!({"name": "Alice", "email": "alice@example.com", "role": "user"}))
            .new_value(json!({"name": "Bob", "email": "alice@example.com", "role": "admin"}))
            .build();

        assert_eq!(details["resource_type"], "user");
        assert_eq!(details["resource_id"], user_id.to_string());

        let changes = details["changes"].as_object().unwrap();
        assert!(changes.contains_key("name"));
        assert!(changes.contains_key("role"));
        assert!(!changes.contains_key("email")); // email didn't change
    }

    #[test]
    fn test_no_old_value() {
        let user_id = Uuid::new_v4();
        let details = AuditChangeBuilder::new("user", user_id)
            .new_value(json!({"name": "Alice"}))
            .build();

        assert!(details["old_value"].is_null());
        assert_eq!(details["new_value"]["name"], "Alice");
        assert!(details["changes"].as_object().unwrap().is_empty());
    }
}
