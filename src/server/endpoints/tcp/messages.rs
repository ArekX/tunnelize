use crate::common::connection::Connection;
use crate::create_channel_enum;

use crate::common::channel::OkResponse;

create_channel_enum!(TcpChannelRequest -> TcpChannelResponse, {
    ClientConnect -> OkResponse
});

#[derive(Debug)]
pub struct ClientConnect {
    pub stream: Option<Connection>,
    pub port: u16,
}
