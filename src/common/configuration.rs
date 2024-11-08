use std::fs::exists;

use serde::{Deserialize, Serialize};

use super::validate::{Validatable, ValidationResult};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerEncryption {
    None,
    Tls { cert_path: String, key_path: String },
}

impl Validatable for ServerEncryption {
    fn validate(&self, result: &mut ValidationResult) {
        if let ServerEncryption::Tls {
            cert_path,
            key_path,
        } = &self
        {
            if !exists(cert_path).is_ok() {
                result.add_error(&format!(
                    "TLS cert path '{}' does not exist or is invalid.",
                    cert_path
                ));
            }

            if !exists(key_path).is_ok() {
                result.add_error(&format!(
                    "TLS key path '{}' does not exist or is invalid.",
                    key_path
                ));
            }
        }
    }
}
