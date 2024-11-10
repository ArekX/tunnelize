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
            "{}: {}",
            self.breadcrumbs.join("."),
            error.to_owned()
        ));
    }

    pub fn add_field_error(&mut self, field: &str, error: &str) {
        self.errors.push(format!(
            "{}{}{}: {}",
            self.breadcrumbs.join("."),
            if self.breadcrumbs.len() > 0 { "." } else { "" },
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
