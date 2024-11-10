use super::validate::{Rule, RuleFor, Validation};

pub struct FileMustExist;

impl Rule for FileMustExist {
    type Value = String;
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

impl Rule for PortMustBeValid {
    type Value = u16;
    fn validate(field: &str, value: &u16, result: &mut Validation) {
        if *value == 0 {
            result.add_field_error(field, "Port cannot be zero.");
        }
    }
}

pub struct HostAddressMustBeValid;

impl Rule for HostAddressMustBeValid {
    type Value = String;
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

impl Rule for IpAddressMustBeValid {
    type Value = String;
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if value.parse::<std::net::IpAddr>().is_err() {
            result.add_field_error(field, "Value must be a valid IP address.");
        }
    }
}

pub struct AlphaNumericOnly;

impl Rule for AlphaNumericOnly {
    type Value = String;
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

impl Rule for IpTemplateMustBeValid {
    type Value = String;
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

        result.validate_rule::<IpAddressMustBeValid>(field, &address.to_owned());
    }
}

pub struct HostnameTemplatemustBeValid;

impl Rule for HostnameTemplatemustBeValid {
    type Value = String;
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if !value.contains("{name}") {
            result.add_field_error(field, "Template must contain the {name} placeholder.");
            return;
        }

        let test_template = value.replace("{port}", "").replace("{name}", "");

        result.validate_rule::<HostAddressMustBeValid>(field, &test_template);
    }
}

pub struct PortHostnameTemplatemustBeValid;

impl Rule for PortHostnameTemplatemustBeValid {
    type Value = String;
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if !value.contains("{port}") {
            result.add_field_error(field, "Template must contain the {port} placeholder.");
            return;
        }

        let test_template = value.replace(":{port}", "");

        result.validate_rule::<HostAddressMustBeValid>(field, &test_template);
    }
}

pub struct UrlTemplateMustBeValid;

impl Rule for UrlTemplateMustBeValid {
    type Value = String;
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if !value.contains("{port}") {
            result.add_field_error(field, "Template must contain the {port} placeholder.");
            return;
        }

        if !value.contains("{hostname}") {
            result.add_field_error(field, "Template must contain the {hostname} placeholder.");
            return;
        }

        let test_template = value.replace("{port}", "").replace("{hostname}", "");

        result.validate_rule::<AlphaNumericOnly>(field, &test_template);
    }
}

pub struct MustBeGreaterThanZero;

impl RuleFor<u16> for MustBeGreaterThanZero {
    fn validate(field: &str, value: &u16, result: &mut Validation) {
        if *value == 0 {
            result.add_field_error(field, "Value must be greater than zero.");
        }
    }
}

impl RuleFor<u64> for MustBeGreaterThanZero {
    fn validate(field: &str, value: &u64, result: &mut Validation) {
        if *value == 0 {
            result.add_field_error(field, "Value must be greater than zero.");
        }
    }
}

impl RuleFor<usize> for MustBeGreaterThanZero {
    fn validate(field: &str, value: &usize, result: &mut Validation) {
        if *value == 0 {
            result.add_field_error(field, "Value must be greater than zero.");
        }
    }
}

pub struct MustNotBeEmptyString;

impl Rule for MustNotBeEmptyString {
    type Value = String;
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if value.trim().is_empty() {
            result.add_field_error(field, "Value cannot be empty.");
        }
    }
}
