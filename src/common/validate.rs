pub trait Validatable {
    fn validate(&self, result: &mut ValidationResult);
}

pub struct ValidationResult {
    prefix_stack: Vec<String>,
    prefix: String,
    errors: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            prefix: "".to_string(),
            prefix_stack: vec![],
        }
    }

    pub fn push_prefix(&mut self, prefix: &str) {
        self.prefix_stack.push(self.prefix.clone());
        self.prefix = format!("{}:", prefix.to_string());
    }

    pub fn pop_prefix(&mut self) {
        self.prefix = match self.prefix_stack.pop() {
            Some(old_prefix) => old_prefix,
            None => "".to_string(),
        };
    }

    pub fn add_error(&mut self, error: &str) {
        self.errors
            .push(format!("{}{}", self.prefix, error.to_owned()));
    }

    pub fn add_field_error(&mut self, field: &str, error: &str) {
        self.errors
            .push(format!("{}{}: {}", self.prefix, field, error));
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }
}

pub fn validate<T: Validatable>(item: &T) -> ValidationResult {
    let mut result = ValidationResult::new();
    item.validate(&mut result);
    result
}
