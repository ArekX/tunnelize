use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpProxy {
    pub desired_name: Option<String>,
    pub forward_address: String,
}
