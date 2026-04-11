//! Shared types used across API and storage layers.

use serde::{Deserialize, Serialize};

/// A schema reference entry (e.g. Protobuf imports, JSON Schema `$ref`).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SchemaReference {
    /// Logical name of the referenced schema (e.g. "User").
    pub name: String,
    /// Subject under which the referenced schema is registered.
    pub subject: String,
    /// Version number of the referenced schema.
    pub version: i32,
}
