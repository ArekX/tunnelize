use serde::{Deserialize, Serialize};

use super::{
    validate::{Validatable, Validation},
    validate_rules::FileMustExist,
};

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
            result.validate_rule::<FileMustExist>("cert_path", cert_path);
            result.validate_rule::<FileMustExist>("key_path", key_path);
        }
    }
}
