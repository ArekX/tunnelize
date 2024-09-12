use std::{collections::HashMap, sync::Arc};

use tokio::{net::TcpStream, sync::Mutex};

pub struct Client {
    pub initial_request: String,
    pub stream: TcpStream,
}

pub type MainClientList = Arc<Mutex<HashMap<u32, Client>>>;

pub fn create_client_list() -> MainClientList {
    Arc::new(Mutex::new(HashMap::new()))
}
