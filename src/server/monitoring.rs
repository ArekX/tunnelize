use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};
use sysinfo::System;

use super::services::{ClientInfo, EndpointInfo, LinkInfo, Services, TunnelInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInfo {
    cpu_count: usize,
    cpu_usages: Vec<String>,
    global_cpu_usage: String,
    available_memory: u64,
    available_memory_percentage: String,
    free_swap: u64,
    free_swap_percentage: String,
    system_name: String,
    kernel_version: String,
    os_version: String,
    hostname: String,
    uptime: String,
    endpoint_count: usize,
    tunnel_count: usize,
    client_count: usize,
    link_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Records<T> {
    pub records: Vec<T>,
}

impl<T> From<Vec<T>> for Records<T> {
    fn from(records: Vec<T>) -> Self {
        Self { records }
    }
}

impl SystemInfo {
    pub async fn new(services: &Arc<Services>) -> Self {
        let sys = System::new_all();
        let cpu_usages = sys
            .cpus()
            .iter()
            .map(|cpu| format!("{:.2}%", cpu.cpu_usage().round()))
            .collect::<Vec<String>>();

        Self {
            cpu_count: sys.cpus().len(),
            cpu_usages,
            global_cpu_usage: format!("{:.2}%", sys.global_cpu_usage().round()),
            available_memory: sys.available_memory(),
            available_memory_percentage: format!(
                "{:.2}%",
                (sys.available_memory() as f64 / sys.total_memory() as f64 * 100f64)
            ),
            free_swap: sys.free_swap(),
            free_swap_percentage: format!(
                "{:.2}%",
                (sys.free_swap() as f64 / sys.total_swap() as f64 * 100f64)
            ),
            system_name: System::name().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            hostname: System::host_name().unwrap_or_default(),
            uptime: services.get_uptime(),
            endpoint_count: services.get_endpoint_manager().await.get_count(),
            tunnel_count: services.get_tunnel_manager().await.get_count(),
            client_count: services.get_client_manager().await.get_count(),
            link_count: services.get_link_manager().await.get_count(),
        }
    }
}

pub async fn get_system_info(services: &Arc<Services>) -> SystemInfo {
    SystemInfo::new(services).await
}

pub async fn get_tunnel_list(services: &Arc<Services>) -> Vec<TunnelInfo> {
    services.get_tunnel_manager().await.list_all_tunnels()
}

pub async fn get_tunnel_info(services: &Arc<Services>, id: &uuid::Uuid) -> Option<TunnelInfo> {
    services.get_tunnel_manager().await.get_tunnel_info(id)
}

pub async fn disconnect_tunnel(services: &Arc<Services>, id: &uuid::Uuid) -> Result<(), String> {
    services.get_tunnel_manager().await.cancel_session(id)
}

pub async fn get_client_list(services: &Arc<Services>) -> Vec<ClientInfo> {
    services.get_client_manager().await.list_all_clients()
}

pub async fn get_client_info(services: &Arc<Services>, id: &uuid::Uuid) -> Option<ClientInfo> {
    services.get_client_manager().await.get_info(id)
}

pub async fn get_endpoint_list(services: &Arc<Services>) -> Vec<EndpointInfo> {
    services.get_endpoint_manager().await.list_endpoints()
}

pub async fn get_endpoint_info(services: &Arc<Services>, name: &str) -> Option<EndpointInfo> {
    services
        .get_endpoint_manager()
        .await
        .get_endpoint_info(name)
}

pub async fn get_link_list(services: &Arc<Services>) -> Vec<LinkInfo> {
    services.get_link_manager().await.list_all_sessions()
}

pub async fn get_link_info(services: &Arc<Services>, id: &uuid::Uuid) -> Option<LinkInfo> {
    services.get_link_manager().await.get_session_info(id)
}

pub async fn disconnect_link(services: &Arc<Services>, id: &uuid::Uuid) -> Result<(), String> {
    services.get_link_manager().await.cancel_session(id)
}
