//! Schema format handling — parsing, canonical form, and fingerprinting.

pub mod avro;
pub mod json_schema;

use crate::error::KoraError;

// -- Types --

/// Supported schema formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaFormat {
    /// Apache Avro schema format.
    Avro,
    /// JSON Schema format.
    Json,
}

/// Parsed and validated schema with computed metadata.
#[derive(Debug)]
pub struct ParsedSchema {
    /// The canonical form of the schema (for deduplication).
    pub canonical_form: String,
    /// Hex-encoded Rabin fingerprint of the canonical form.
    pub fingerprint: String,
}

// -- Functions --

impl SchemaFormat {
    /// Known schema types advertised by the registry (matches Confluent).
    pub const KNOWN_TYPES: &[&str] = &["AVRO", "JSON", "PROTOBUF"];

    /// Parse a format string, defaulting to Avro when `None`.
    ///
    /// # Errors
    ///
    /// Returns `KoraError::InvalidSchema` for unrecognized formats.
    pub fn from_optional(schema_type: Option<&str>) -> Result<Self, KoraError> {
        match schema_type.map(str::to_ascii_uppercase).as_deref() {
            None | Some("AVRO") => Ok(Self::Avro),
            Some("JSON") => Ok(Self::Json),
            Some(other) => Err(KoraError::InvalidSchema(format!(
                "Unsupported schema type: {other}"
            ))),
        }
    }

    /// Wire-format name used in database and API responses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Avro => "AVRO",
            Self::Json => "JSON",
        }
    }
}

/// Parse and validate a raw schema string.
///
/// # Errors
///
/// Returns `KoraError::InvalidSchema` if the schema is malformed.
pub fn parse(format: SchemaFormat, raw: &str) -> Result<ParsedSchema, KoraError> {
    match format {
        SchemaFormat::Avro => avro::parse(raw),
        SchemaFormat::Json => json_schema::parse(raw),
    }
}
