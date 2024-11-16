use super::validate::{Rule, RuleFor, Validation};

pub struct FileMustExist;

impl Rule for FileMustExist {
    type Value = String;
    fn validate(field: &str, value: &String, result: &mut Validation) {
        if value.is_empty() {
            result.add_field_error(field, "Value cannot be empty.");
            return;
        }

        let file_exists = match std::fs::exists(value) {
            Ok(result) => result,
            Err(_) => false,
        };

        if !file_exists {
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

        let test_template = value.replace(":{port}", "").replace("{hostname}", "");

        let test_template = test_template
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .to_string();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::validate::Validation;

    fn assert_field_error(validation: &Validation, field: &str, expected_error: &str) {
        let errors = validation.errors();

        let mut found = false;
        let check = format!("{}: {}", field, expected_error);
        for error in errors {
            if error.contains(&check) {
                found = true;
                break;
            }
        }

        assert!(found);
    }

    #[test]
    fn test_file_must_exist_invalid() {
        let mut validation = Validation::new();
        FileMustExist::validate("file", &"".to_string(), &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(&validation, "file", "Value cannot be empty.");

        validation = Validation::new();
        FileMustExist::validate("file", &"/invalid/path".to_string(), &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(
            &validation,
            "file",
            "File '/invalid/path' does not exist or is invalid.",
        );
    }

    #[test]
    fn test_file_must_exist_valid() {
        let mut validation = Validation::new();
        let exe_name = std::env::current_exe()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        FileMustExist::validate("file", &exe_name, &mut validation);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_port_must_be_valid_invalid() {
        let mut validation = Validation::new();
        PortMustBeValid::validate("port", &0, &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(&validation, "port", "Port cannot be zero.");
    }

    #[test]
    fn test_port_must_be_valid_valid() {
        let mut validation = Validation::new();
        PortMustBeValid::validate("port", &8080, &mut validation);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_host_address_must_be_valid_invalid() {
        let mut validation = Validation::new();
        HostAddressMustBeValid::validate("host", &"invalid host!".to_string(), &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(
            &validation,
            "host",
            "Can only contain alphanumeric characters, hyphens, and periods.",
        );
    }

    #[test]
    fn test_host_address_must_be_valid_valid() {
        let mut validation = Validation::new();
        HostAddressMustBeValid::validate("host", &"valid-host.com".to_string(), &mut validation);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_ip_address_must_be_valid_invalid() {
        let mut validation = Validation::new();
        IpAddressMustBeValid::validate("ip", &"invalid_ip".to_string(), &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(&validation, "ip", "Value must be a valid IP address.");
    }

    #[test]
    fn test_ip_address_must_be_valid_valid() {
        let mut validation = Validation::new();
        IpAddressMustBeValid::validate("ip", &"192.168.1.1".to_string(), &mut validation);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_alpha_numeric_only_invalid() {
        let mut validation = Validation::new();
        AlphaNumericOnly::validate("alpha", &"invalid@chars".to_string(), &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(
            &validation,
            "alpha",
            "Can only contain alphanumeric characters and hyphens.",
        );
    }

    #[test]
    fn test_alpha_numeric_only_valid() {
        let mut validation = Validation::new();
        AlphaNumericOnly::validate("alpha", &"valid-chars".to_string(), &mut validation);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_ip_template_must_be_valid_invalid() {
        let mut validation = Validation::new();
        IpTemplateMustBeValid::validate(
            "template",
            &"invalid_template".to_string(),
            &mut validation,
        );
        assert!(!validation.is_valid());
        assert_field_error(
            &validation,
            "template",
            "Template must contain the {port} placeholder.",
        );
    }

    #[test]
    fn test_ip_template_must_be_valid_valid() {
        let mut validation = Validation::new();
        IpTemplateMustBeValid::validate(
            "template",
            &"192.168.1.1:{port}".to_string(),
            &mut validation,
        );
        assert!(validation.is_valid());
    }

    #[test]
    fn test_hostname_template_must_be_valid_invalid() {
        let mut validation = Validation::new();
        HostnameTemplatemustBeValid::validate(
            "template",
            &"invalid_template".to_string(),
            &mut validation,
        );
        assert!(!validation.is_valid());
        assert_field_error(
            &validation,
            "template",
            "Template must contain the {name} placeholder.",
        );
    }

    #[test]
    fn test_hostname_template_must_be_valid_valid() {
        let mut validation = Validation::new();
        HostnameTemplatemustBeValid::validate(
            "template",
            &"host-{name}".to_string(),
            &mut validation,
        );
        assert!(validation.is_valid());
    }

    #[test]
    fn test_port_hostname_template_must_be_valid_invalid() {
        let mut validation = Validation::new();
        PortHostnameTemplatemustBeValid::validate(
            "template",
            &"invalid_template".to_string(),
            &mut validation,
        );
        assert!(!validation.is_valid());
        assert_field_error(
            &validation,
            "template",
            "Template must contain the {port} placeholder.",
        );
    }

    #[test]
    fn test_port_hostname_template_must_be_valid_valid() {
        let mut validation = Validation::new();
        PortHostnameTemplatemustBeValid::validate(
            "template",
            &"host:{port}".to_string(),
            &mut validation,
        );
        assert!(validation.is_valid());
    }

    #[test]
    fn test_url_template_must_be_valid_invalid() {
        let mut validation = Validation::new();
        UrlTemplateMustBeValid::validate(
            "template",
            &"invalid_template".to_string(),
            &mut validation,
        );
        assert!(!validation.is_valid());
        assert_field_error(
            &validation,
            "template",
            "Template must contain the {port} placeholder.",
        );
    }

    #[test]
    fn test_url_template_must_be_valid_valid() {
        let mut validation = Validation::new();
        UrlTemplateMustBeValid::validate(
            "template",
            &"http://{hostname}:{port}".to_string(),
            &mut validation,
        );
        assert!(validation.is_valid());
    }

    #[test]
    fn test_must_be_greater_than_zero_invalid() {
        let mut validation = Validation::new();
        MustBeGreaterThanZero::validate("value", &0u16, &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(&validation, "value", "Value must be greater than zero.");

        validation = Validation::new();
        MustBeGreaterThanZero::validate("value", &0u64, &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(&validation, "value", "Value must be greater than zero.");

        validation = Validation::new();
        MustBeGreaterThanZero::validate("value", &0usize, &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(&validation, "value", "Value must be greater than zero.");
    }

    #[test]
    fn test_must_be_greater_than_zero_valid() {
        let mut validation = Validation::new();
        MustBeGreaterThanZero::validate("value", &1u16, &mut validation);
        assert!(validation.is_valid());

        validation = Validation::new();
        MustBeGreaterThanZero::validate("value", &1u64, &mut validation);
        assert!(validation.is_valid());

        validation = Validation::new();
        MustBeGreaterThanZero::validate("value", &1usize, &mut validation);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_must_not_be_empty_string_invalid() {
        let mut validation = Validation::new();
        MustNotBeEmptyString::validate("value", &"".to_string(), &mut validation);
        assert!(!validation.is_valid());
        assert_field_error(&validation, "value", "Value cannot be empty.");
    }

    #[test]
    fn test_must_not_be_empty_string_valid() {
        let mut validation = Validation::new();
        MustNotBeEmptyString::validate("value", &"non-empty".to_string(), &mut validation);
        assert!(validation.is_valid());
    }
}
