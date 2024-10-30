use crate::common::connection::Connection;
use crate::common::data_bridge::UdpSession;
use crate::create_channel_enum;

use crate::common::channel::OkResponse;

create_channel_enum!(UdpChannelRequest -> UdpChannelResponse, {
    ClientConnect -> OkResponse
});

#[derive(Debug)]
pub struct ClientConnect {
    pub initial_data: Option<Vec<u8>>,
    pub stream: Option<Connection>,
    pub session: Option<UdpSession>,
    pub port: u16,
}
