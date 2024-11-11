use std::fs::exists;

use serde::{Deserialize, Serialize};

use super::validate::{Validatable, Validation};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEncryption {
    None,
    Tls { cert_path: String, key_path: String },
}

impl Validatable for ServerEncryption {
    fn validate(&self, result: &mut Validation) {
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
