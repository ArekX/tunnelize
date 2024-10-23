use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use serde_json::json;
use sysinfo::System;

use super::{response::into_json, state::AppState};

async fn get_system_info(State(state): State<AppState>) -> impl IntoResponse {
    let sys = System::new_all();

    let cpu_usages = sys
        .cpus()
        .iter()
        .map(|cpu| format!("{}%", cpu.cpu_usage().round()))
        .collect::<Vec<String>>();

    into_json(
        StatusCode::OK,
        json!({
            "cpu_count": sys.cpus().len(),
            "cpu_usages": cpu_usages,
            "global_cpu_usage": format!("{}%", sys.global_cpu_usage().round()),
            "available_memory": sys.available_memory(),
            "free_memory_percentage": (sys.available_memory() as f64 / sys.total_memory() as f64 * 100f64).round(),
            "free_swap": sys.total_swap() - sys.used_swap(),
            "system_name": System::name(),
            "kernel_version": System::kernel_version(),
            "os_version": System::os_version(),
            "hostname": System::host_name(),
            "uptime": state.get_uptime(),
            "endpoint_count": state.services.get_endpoint_manager().await.get_count(),
            "tunnel_count": state.services.get_tunnel_manager().await.get_count(),
            "link_count": state.services.get_link_manager().await.get_count(),
        }),
    )
}

pub fn get_router() -> Router<AppState> {
    Router::new().route("/info", get(get_system_info))
}
