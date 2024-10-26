use crate::common::connection::ConnectionStream;
use crate::create_channel_enum;

use crate::common::channel::OkResponse;

create_channel_enum!(UdpChannelRequest -> UdpChannelResponse, {
    ClientConnect -> OkResponse
});

#[derive(Debug)]
pub struct ClientConnect {
    pub initial_data: Option<Vec<u8>>,
    pub stream: Option<ConnectionStream>,
    pub port: u16,
}
