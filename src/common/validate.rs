pub trait Validatable {
    fn validate(&self, result: &mut Validation);
}

pub trait Rule {
    type Value;
    fn validate(field: &str, value: &Self::Value, result: &mut Validation);
}

pub trait RuleFor<Value> {
    fn validate(field: &str, value: &Value, result: &mut Validation);
}

pub struct Validation {
    breadcrumbs: Vec<String>,
    errors: Vec<String>,
}

impl Validation {
    pub fn validate(item: &impl Validatable) -> Validation {
        let mut instance = Validation::new();

        item.validate(&mut instance);

        instance
    }

    pub fn new() -> Self {
        Self {
            errors: vec![],
            breadcrumbs: vec![],
        }
    }

    pub fn validate_rule_for<Value, Rule>(&mut self, field: &str, value: &Value)
    where
        Rule: RuleFor<Value>,
    {
        Rule::validate(field, value, self);
    }

    pub fn validate_rule<RuleType>(&mut self, field: &str, value: &RuleType::Value)
    where
        RuleType: Rule,
    {
        RuleType::validate(field, value, self);
    }

    pub fn validate_child(&mut self, breadcrumb: &str, item: &impl Validatable) {
        self.push_breadcrumb(breadcrumb);
        item.validate(self);
        self.pop_breadcrumb();
    }

    pub fn push_breadcrumb(&mut self, prefix: &str) {
        self.breadcrumbs.push(prefix.to_owned());
    }

    pub fn pop_breadcrumb(&mut self) {
        self.breadcrumbs.pop();
    }

    pub fn add_error(&mut self, error: &str) {
        self.errors.push(format!(
            "{}{}",
            if self.breadcrumbs.is_empty() {
                "".to_owned()
            } else {
                format!("{}: ", self.breadcrumbs.join("."))
            },
            error.to_owned()
        ));
    }

    pub fn add_field_error(&mut self, field: &str, error: &str) {
        self.errors.push(format!(
            "{}{}{}: {}",
            self.breadcrumbs.join("."),
            if !self.breadcrumbs.is_empty() { "." } else { "" },
            field,
            error.to_owned()
        ));
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestValidatable {
        is_valid: bool,
    }

    impl Validatable for TestValidatable {
        fn validate(&self, result: &mut Validation) {
            if !self.is_valid {
                result.add_error("TestValidatable is invalid");
            }
        }
    }

    struct TestRule;

    impl Rule for TestRule {
        type Value = bool;

        fn validate(field: &str, value: &Self::Value, result: &mut Validation) {
            if !value {
                result.add_field_error(field, "TestRule validation failed");
            }
        }
    }

    struct TestRuleFor;

    impl RuleFor<bool> for TestRuleFor {
        fn validate(field: &str, value: &bool, result: &mut Validation) {
            if !value {
                result.add_field_error(field, "TestRuleFor validation failed");
            }
        }
    }

    #[test]
    fn test_validatable() {
        let valid_item = TestValidatable { is_valid: true };
        let invalid_item = TestValidatable { is_valid: false };

        let valid_result = Validation::validate(&valid_item);
        let invalid_result = Validation::validate(&invalid_item);

        assert!(valid_result.is_valid());
        assert!(!invalid_result.is_valid());
        assert_eq!(invalid_result.errors(), &vec!["TestValidatable is invalid"]);
    }

    #[test]
    fn test_rule() {
        let mut validation = Validation::new();
        validation.validate_rule::<TestRule>("test_field", &true);
        assert!(validation.is_valid());
        validation.validate_rule::<TestRule>("test_field", &false);

        assert!(!validation.is_valid());
        assert_eq!(validation.errors().len(), 1);
        assert_eq!(
            validation.errors()[0],
            "test_field: TestRule validation failed"
        );
    }

    #[test]
    fn test_rule_for() {
        let mut validation = Validation::new();
        validation.validate_rule_for::<bool, TestRuleFor>("test_field", &true);
        assert!(validation.is_valid());
        validation.validate_rule_for::<bool, TestRuleFor>("test_field", &false);

        assert!(!validation.is_valid());
        assert_eq!(validation.errors().len(), 1);
        assert_eq!(
            validation.errors()[0],
            "test_field: TestRuleFor validation failed"
        );
    }

    #[test]
    fn test_validate_child() {
        let parent = TestValidatable { is_valid: true };
        let child = TestValidatable { is_valid: false };

        let mut validation = Validation::new();
        validation.validate_child("parent", &parent);
        validation.validate_child("child", &child);

        assert!(!validation.is_valid());
        assert_eq!(validation.errors().len(), 1);
        assert_eq!(validation.errors()[0], "child: TestValidatable is invalid");
    }
}
