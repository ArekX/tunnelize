use super::validate::{Rule, Validation};

pub struct FileMustExist;

impl Rule<String> for FileMustExist {
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if !std::fs::exists(value).is_ok() {
            result.add_field_error(
                field,
                &format!("File '{}' does not exist or is invalid.", value),
            );
        }
    }
}

pub struct PortMustBeValid;

impl Rule<u16> for PortMustBeValid {
    fn validate(field: &str, value: &u16, result: &mut Validation) {
        if *value == 0 {
            result.add_field_error(field, "Port cannot be zero.");
        }
    }
}

pub struct HostAddressMustBeValid;

impl Rule<String> for HostAddressMustBeValid {
    fn validate(field: &str, value: &String, result: &mut Validation) {
        // TODO: Implement this rule.
    }
}

pub struct IpAddressMustBeValid;

impl Rule<String> for IpAddressMustBeValid {
    fn validate(field: &str, value: &String, result: &mut Validation) {
        // TODO: Implement this rule.
    }
}

pub struct AlphaNumericOnly;

impl Rule<String> for AlphaNumericOnly {
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if !value.chars().all(|c| c.is_alphanumeric() || c == '-') {
            result.add_field_error(field, "Value must be alphanumeric.");
        }
    }
}
