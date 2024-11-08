pub trait Validatable {
    fn validate(&self, result: &mut ValidationResult);
}

pub struct ValidationResult {
    breadcrumbs: Vec<String>,
    errors: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            breadcrumbs: vec![],
        }
    }

    pub fn with_breadcrumb<T: FnOnce(&mut ValidationResult)>(&mut self, breadcrumb: &str, f: T) {
        self.push_breadcrumb(breadcrumb);
        f(self);
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
            "{}{}{}:{}",
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

pub fn validate<T: Validatable>(item: &T) -> ValidationResult {
    let mut result = ValidationResult::new();
    item.validate(&mut result);
    result
}
