use super::validate::{Rule, StatefulRule, Validation};

pub struct FileMustExist;

impl Rule<String> for FileMustExist {
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if value.is_empty() {
            result.add_field_error(field, "Value cannot be empty.");
        } else if !std::fs::exists(value).is_ok() {
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
        if !value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '.')
        {
            result.add_field_error(
                field,
                "Can only contain alphanumeric characters, hyphens, and periods.",
            );
        }
    }
}

pub struct IpAddressMustBeValid;

impl Rule<String> for IpAddressMustBeValid {
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if value.parse::<std::net::IpAddr>().is_err() {
            result.add_field_error(field, "Value must be a valid IP address.");
        }
    }
}

pub struct AlphaNumericOnly;

impl Rule<String> for AlphaNumericOnly {
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if !value.chars().all(|c| c.is_alphanumeric() || c == '-') {
            result.add_field_error(
                field,
                "Can only contain alphanumeric characters and hyphens.",
            );
        }
    }
}

pub struct IpTemplateMustBeValid;

impl Rule<String> for IpTemplateMustBeValid {
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if !value.contains("{port}") {
            result.add_field_error(field, "Template must contain the {port} placeholder.");
            return;
        }

        let test_template = value.replace("{port}", "1234");
        let (address, _) = test_template
            .split_once(':')
            .unwrap_or_else(|| ("", ""))
            .to_owned();

        result.validate_rule::<IpAddressMustBeValid, String>("address", &address.to_owned());
    }
}

pub struct MustBeGreaterThanZero;

impl Rule<u16> for MustBeGreaterThanZero {
    fn validate(field: &str, value: &u16, result: &mut Validation) {
        if *value == 0 {
            result.add_field_error(field, "Value must be greater than zero.");
        }
    }
}

impl Rule<u64> for MustBeGreaterThanZero {
    fn validate(field: &str, value: &u64, result: &mut Validation) {
        if *value == 0 {
            result.add_field_error(field, "Value must be greater than zero.");
        }
    }
}

pub struct MustBeBetween(u16, u16);

impl StatefulRule<u16> for MustBeBetween {
    fn validate(&self, field: &str, value: &u16, result: &mut Validation) {
        if *value < self.0 || *value > self.1 {
            result.add_field_error(
                field,
                &format!("Value must be between {} and {}.", self.0, self.1),
            );
        }
    }
}
