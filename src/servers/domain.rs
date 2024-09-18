use std::{collections::HashSet, sync::Arc};

use rand::Rng;
use tokio::sync::Mutex;

const CHARACTER_SET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARACTER_SET.len());
            CHARACTER_SET[idx] as char
        })
        .collect()
}

pub struct DomainRegistrar {
    pub template: String,
    pub domain_map: HashSet<String>,
}

impl DomainRegistrar {
    pub fn new(template: String) -> Self {
        DomainRegistrar {
            template,
            domain_map: HashSet::new(),
        }
    }

    pub fn register_domain(&mut self, preferred_name: Option<String>) -> String {
        if let Some(name) = preferred_name {
            let domain = self.template.replace("{dynamic}", &name);

            if !self.domain_map.contains(&domain) {
                self.domain_map.insert(domain.clone());
                return domain;
            }
        }

        let mut domain = self
            .template
            .replace("{dynamic}", &generate_random_string(8));

        while self.domain_map.contains(&domain) {
            domain = self
                .template
                .replace("{dynamic}", &generate_random_string(8));
        }

        self.domain_map.insert(domain.clone());
        domain
    }
}

pub type RegistrarList = Arc<Mutex<DomainRegistrar>>;

pub fn create_registrar_list(template: String) -> RegistrarList {
    Arc::new(Mutex::new(DomainRegistrar::new(template)))
}
