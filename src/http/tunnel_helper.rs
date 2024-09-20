use super::{host_list::HostList, tunnel_list::TunnelList, TaskService};

pub async fn disconnect_tunnel(
    host_service: &TaskService<HostList>,
    tunnel_service: &TaskService<TunnelList>,
    tunnel_id: u32,
) {
    host_service.lock().await.unregister_by_tunnel(tunnel_id);
    tunnel_service.lock().await.remove_tunnel(tunnel_id);
}
